use core::fmt::Debug;
use core::marker::PhantomData;
use core::mem::size_of;

use alloc::vec::Vec;

use shim::io;
use shim::ioerr;
use shim::newioerr;
use shim::path;
use shim::path::Path;

use crate::mbr::MasterBootRecord;
use crate::traits::{BlockDevice, FileSystem};

use crate::util::SliceExt;
use crate::vfat::{BiosParameterBlock, CachedPartition, Partition};
// use crate::vfat::{Cluster, Dir, Entry, Error, FatEntry, File, Status};
use crate::vfat::{Error, Cluster};

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
        let mbr = MasterBootRecord::from(&mut device);

        let start_of_partition = mbr.partition_1.relative_sector;


        //consider changing to work if the first partition isnt FAT (loop over partition entries)
        let bpb = BiosParameterBlock::from(&mut device, start_of_partition);

        let BiosParameterBlock { num_sectors, bytes_per_sector, sectors_per_cluster, sectors_per_fat, num_reserved_sectors, num_FATs, root_dir_cluster, ..  } = bpb;
        // let num_sectors = bpb.total_logical_sectors;
        // let bytes_per_sector = bpb.bytes_per_sector;
        // let sectors_per_cluster = bpb.sectors_per_clsuter;
        // let sectors_per_fat = bpb.sectors_per_fat;
        // let num_reserved_sectors = bpb.num_reserved_sectors;
        // let num_FATs = bpb.num_FATs;
        // let root_dir_cluster = bpb.root_dir_cluster;

        let fat_start_sector = start_of_partition + 1 + num_reserved_sectors;
        let data_start_sector = fat_start_sector + num_FATs * sectors_per_fat;

        let partition = Partition { 
                                    start: start_of_partition, 
                                    num_sectors, 
                                    sector_size: bytes_per_sector
                                  };

        let cached_partition = CachedPartition::new(device, partition);

        let vfat = VFat {
                        device: cached_partition,
                        bytes_per_sector,
                        sectors_per_cluster,
                        sectors_per_fat,
                        fat_start_sector,
                        data_start_sector,
                        root_dir_cluster: root_dir_cluster as u32
                        };
                                
    }

    // TODO: The following methods may be useful here:
    //
    //  * A method to read from an offset of a cluster into a buffer.
    //
    //    fn read_cluster(
    //        &mut self,
    //        cluster: Cluster,
    //        offset: usize,
    //        buf: &mut [u8]
    //    ) -> io::Result<usize>;
    //
    //  * A method to read all of the clusters chained from a starting cluster
    //    into a vector.
    //
    //    fn read_chain(
    //        &mut self,
    //        start: Cluster,
    //        buf: &mut Vec<u8>
    //    ) -> io::Result<usize>;
    //
    //  * A method to return a reference to a `FatEntry` for a cluster where the
    //    reference points directly into a cached sector.
    //
    //    fn fat_entry(&mut self, cluster: Cluster) -> io::Result<&FatEntry>;
}

impl<'a, HANDLE: VFatHandle> FileSystem for &'a HANDLE {
    type File = crate::traits::Dummy;
    type Dir = crate::traits::Dummy;
    type Entry = crate::traits::Dummy;

    fn open<P: AsRef<Path>>(self, path: P) -> io::Result<Self::Entry> {
        unimplemented!("FileSystem::open()")
    }
}
