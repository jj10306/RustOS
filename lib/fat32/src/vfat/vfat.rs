use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;
use alloc::string::String;

use shim::const_assert_size;
use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::{Path, Component, Components, PathBuf};

use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem, Entry as EntryTrait};

use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status};


/// A generic trait that handles a critical section as a closure
pub trait VFatHandle: Clone + Debug + Send + Sync {
    fn new(val: VFat<Self>) -> Self;
    fn lock<R>(&self, f: impl FnOnce(&mut VFat<Self>) -> R) -> R;
}

#[derive(Debug)]
pub struct VFat<HANDLE: VFatHandle> {
    phantom: PhantomData<HANDLE>,
    device: CachedPartition,
    bytes_per_sector: u16, //bpb.bytes_per_sector
    sectors_per_cluster: u8,//bpb.sectors_per_clsuter
    sectors_per_fat: u32, //bpb.sectors_per_fat
    fat_start_sector: u64, //mbr.partition_1.relative_sector
    data_start_sector: u64, // 
    rootdir_cluster: Cluster,
}

impl<HANDLE: VFatHandle> VFat<HANDLE> {
    pub fn from<T>(mut device: T) -> Result<HANDLE, Error>
    where
        T: BlockDevice + 'static,
    {
        let mbr = MasterBootRecord::from(&mut device)?;

        // consider changing  to work if the first partition isnt FAT (loop over partition entries in MBR?)
        let start_of_partition;
        if mbr.partition_table_entry_1.partition_type == 0xB || mbr.partition_table_entry_1.partition_type == 0xC {
            start_of_partition = mbr.partition_table_entry_1.relative_sector as u64;
        } else if mbr.partition_table_entry_2.partition_type == 0xB || mbr.partition_table_entry_2.partition_type == 0xC {
            start_of_partition = mbr.partition_table_entry_2.relative_sector as u64;
        } else if mbr.partition_table_entry_3.partition_type == 0xB || mbr.partition_table_entry_3.partition_type == 0xC {
            start_of_partition = mbr.partition_table_entry_3.relative_sector as u64;
        } else if mbr.partition_table_entry_4.partition_type == 0xB || mbr.partition_table_entry_4.partition_type == 0xC {
            start_of_partition = mbr.partition_table_entry_4.relative_sector as u64;
        } else {
            panic!("No FAT partition found");
        }
        let bpb = BiosParameterBlock::from(&mut device, start_of_partition)?;

        let BiosParameterBlock { total_logical_sectors: num_sectors, total_logical_sectors_alt: num_sectors_alt, bytes_per_sector, sectors_per_cluster, sectors_per_FAT: sectors_per_fat, num_reserved_sectors, num_FATs, root_cluster_num, ..  } = bpb;
        // let num_sectors = bpb.total_logical_sectors;
        // let bytes_per_sector = bpb.bytes_per_sector;
        // let sectors_per_cluster = bpb.sectors_per_clsuter;
        // let sectors_per_fat = bpb.sectors_per_fat;
        // let num_reserved_sectors = bpb.num_reserved_sectors;
        // let num_FATs = bpb.num_FATs;
        // let root_dir_cluster = bpb.root_dir_cluster;

        //should these be relative to the start of the partition or the start of the disk itself, in other words
        // should it be: let fat_start_sector = start_of_partition + num_reserved_sectors;
        let fat_start_sector = num_reserved_sectors;
        let data_start_sector = fat_start_sector as u64 + (num_FATs as u32 * sectors_per_fat) as u64;
        let num_logical_sectors = {
            if num_sectors == 0 {
                num_sectors_alt as u64
            } else {
                num_sectors as u64
            }
        };
        let partition = Partition { 
                                    start: start_of_partition, 
                                    num_sectors: num_logical_sectors, 
                                    sector_size: bytes_per_sector as u64
                                  };

        let cached_partition = CachedPartition::new(device, partition);

        let vfat = VFat {
                        phantom: PhantomData,
                        device: cached_partition,
                        bytes_per_sector,
                        sectors_per_cluster,
                        sectors_per_fat,
                        fat_start_sector: fat_start_sector as u64,
                        data_start_sector: data_start_sector,
                        rootdir_cluster: Cluster::from(root_cluster_num)
                        };
        Ok(VFatHandle::new(vfat))

    }

    // TODO: The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.
    //
       pub fn read_cluster(
           &mut self,
           cluster: Cluster,
           offset: usize,
           buf: &mut [u8]
       ) -> io::Result<usize> {

           //consider checking if the cluster is valid for reading??

        // if buf.len() % self.bytes_per_sector as usize != 0 {
        //     ioerr!(InvalidInput, "Buff length must be a positive multiple of bytes per sector")
        // } 
        if offset > self.sectors_per_cluster as usize {
            ioerr!(InvalidInput, "Offset must be <= to sectors per clusters")
        } else {
            // get the sector number for the starting sector of the cluster
            let cluster_sector_number = cluster.sector_from_cluster(self.data_start_sector, self.sectors_per_cluster as u64) + offset as u64;  
            // determine the limiting factor in how many sectors to read
            // either the buffers length or the remaining sectors in the cluster after offset
            let sectors_to_read = core::cmp::min(buf.len() / self.bytes_per_sector as usize, self.sectors_per_cluster as usize - offset);
            // let mut vec_buf = vec![0u8; sectors_to_read * self.bytes_per_sector as usize];
            let mut vec_buf = Vec::new();
            for sector in cluster_sector_number..cluster_sector_number + sectors_to_read as u64 {
                let sector_slice = self.device.get(sector)?;
                vec_buf.extend_from_slice(sector_slice);
            }

            buf.clone_from_slice(&vec_buf[..buf.len()]);
            //returning the number of bytes read, could also just do the number of sectosrs
            Ok(sectors_to_read * self.bytes_per_sector as usize)
        }
       }
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
       pub fn read_chain(
           &mut self,
           start: Cluster,
           buf: &mut Vec<u8>
       ) -> io::Result<usize> {
           
        //consider checking if this is a valid 
        if false {
            ioerr!(InvalidInput, "Cluster number must be greater than 2")
        } else {
            let mut bytes_read = 0;
            let mut current_cluster = start;
            loop {
                buf.resize(buf.len() + self.bytes_per_sector as usize * self.sectors_per_cluster as usize,0);
                match self.fat_entry(current_cluster)?.status() {
                    Status::Data(next_cluster) => {
                        bytes_read += self.read_cluster(current_cluster,0, &mut buf[bytes_read..])?;
                        current_cluster = next_cluster;
                    },
                    Status::Eoc(_) => {
                        bytes_read += self.read_cluster(current_cluster,0, &mut buf[bytes_read..])?;
                        return Ok(bytes_read);
                    },
                    _ => {
                        return ioerr!(Other, "Reserved or cluster or bad sector encountered in chain");
                    }
                }
            }
        }
       }
    //    fn read_into_chain_buffer(&mut self, curr: Cluster, buf: &mut Vec<u8>) -> io::Result<usize> {
    //         let bytes_in_cluster = self.sectors_per_cluster as usize * self.bytes_per_sector as usize; 
    //         let mut read_buf = vec![0u8; bytes_in_cluster];
    //         // let mut read_buf = Vec::new();
    //         let bytes_read = self.read_cluster(curr, 0, read_buf.as_mut_slice())?;
    //         if bytes_read != bytes_in_cluster {
    //             return ioerr!(Other, "bytes read don't match total bytes in cluster");
    //         }
    //         buf.extend_from_slice(read_buf.as_slice());
    //         Ok(bytes_read)
    //    }

    //
    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
       fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry> {
           // multiply cluster number by 4 (32 bits, 4 bytes) to get byte offset from start of FAT
            let fat_offset = (cluster.get_cluster_number() * size_of::<FatEntry>() as u32) as u64;
            // convert offset to sectors and add that to the fat_start_sector to ignore reserved sectors
            let fat_sec_num = self.fat_start_sector + (fat_offset / self.bytes_per_sector as u64);
            // good chance that the offset isn't perfectly a multiple of 'bytes_per_sector', so must get the remainder to get the offset from the sector number computer above
            let fat_entry_offset = (fat_offset % self.bytes_per_sector as u64) as usize;
            
            let fat_sec_slice = self.device.get(fat_sec_num)?;

            unsafe {
                // let fat_entries: &[FatEntry] = core::mem::transmute(fat_sec_slice);
                let fat_entries: &[FatEntry] = fat_sec_slice.cast() ;
                Ok(&fat_entries[fat_entry_offset as usize / size_of::<FatEntry>()])
            }
    // let entries_per_fat = self.bytes_per_sector as u64 / size_of::<FatEntry>() as u64;
    // let fat_index = cluster.get_cluster_number() as u64 / entries_per_fat;
    // let fat_offset = cluster.get_cluster_number() as u64 % entries_per_fat;

    // let data = self.device.get(self.fat_start_sector + fat_index)?;
    // let entries: &[FatEntry] = unsafe { core::mem::transmute(data) };

    // return Ok(&entries[fat_offset as usize]);


       }
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Entry = Entry<HANDLE>;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        //resolve '.' and '..' in path bc '..' was causing UB
        let mut resolved_path = PathBuf::new();
        let mut comps = path.as_ref().components();
        for comp in comps {
            match comp {
                Component::RootDir => {resolved_path.push("/");},
                Component::Normal(s) => {resolved_path.push(s);},
                Component::ParentDir => {resolved_path.pop();}
                _ => {;}
            }
        }
        // panic!("{:?}", resolved_path.into_os_string().into_string().unwrap());

        let root_dir_cluster = self.lock(|fat| fat.rootdir_cluster);

        let root_dir = Dir::new(self.clone(), root_dir_cluster, String::from("/"));
        let mut entry = Entry::Dir(root_dir);
        let mut components = resolved_path.as_path().components();
        let mut component = components.next();
        //skip root dir '/'
        match component.expect("Oops iterator") {
            Component::RootDir => {component = components.next();},
            _ => {return ioerr!(Other, "Expected root, found something else") ;}
        }
        while component.is_some() {
            let component_string = match component.unwrap() {
                Component::Normal(string) => string.to_str().unwrap(),
                Component::ParentDir => "..",
                Component::CurDir => ".",
                _ => panic!("this should never happen ah")
            };

            entry = entry.into_dir().expect("Error converting to Dir").find(component_string)?;
            // if a == 0 {
            //     panic!("{:?}", entry);
            // }
            match entry {
                Entry::Dir(dir) => {entry = Entry::Dir(dir);},
                Entry::File(_) => { 
                    if components.next() == None {
                        break;
                    }
                    return ioerr!(NotFound, "Invalid path");
            }
            }
            component = components.next();
        }
        Ok(entry)
    }
}
