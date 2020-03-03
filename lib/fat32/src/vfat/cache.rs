use alloc::boxed::Box;
use alloc::vec::Vec;
use core::fmt;
use hashbrown::HashMap;
use hashbrown::hash_map::Entry;
use shim::io;

use crate::traits::BlockDevice;

#[derive(Debug)]
struct CacheEntry {
    data: Vec<u8>,
    dirty: bool,
}

pub struct Partition {
    /// The physical sector where the partition begins.     
    pub start: u64,
    /// Number of sectors
    pub num_sectors: u64,
    /// The size, in bytes, of a logical sector in the partition.
    pub sector_size: u64,
}

pub struct CachedPartition {
    device: Box<dyn BlockDevice>,
    cache: HashMap<u64, CacheEntry>,
    partition: Partition,
}

impl CachedPartition {
    /// Creates a new `CachedPartition` that transparently caches sectors from
    /// `device` and maps physical sectors to logical sectors inside of
    /// `partition`. All reads and writes from `CacheDevice` are performed on
    /// in-memory caches.
    ///
    /// The `partition` parameter determines the size of a logical sector and
    /// where logical sectors begin. An access to a sector `0` will be
    /// translated to physical sector `partition.start`. Virtual sectors of
    /// sector number `[0, num_sectors)` are accessible.
    ///
    /// `partition.sector_size` must be an integer multiple of
    /// `device.sector_size()`.
    ///
    /// # Panics
    ///
    /// Panics if the partition's sector size is < the device's sector size.
    pub fn new<T>(device: T, partition: Partition) -> CachedPartition
    where
        T: BlockDevice + 'static,
    {
        assert!(partition.sector_size >= device.sector_size());

        CachedPartition {
            device: Box::new(device),
            cache: HashMap::new(),
            partition: partition,
        }
    }

    /// Returns the number of physical sectors that corresponds to
    /// one logical sector.
    fn factor(&self) -> u64 {
        self.partition.sector_size / self.device.sector_size()
    }

    /// Maps a user's request for a sector `virt` to the physical sector.
    /// Returns `None` if the virtual sector number is out of range.
    fn virtual_to_physical(&self, virt: u64) -> Option<u64> {
        if virt >= self.partition.num_sectors {
            return None;
        }

        let physical_offset = virt * self.factor();
        let physical_sector = self.partition.start + physical_offset;

        Some(physical_sector)
    }

    /// Returns a mutable reference to the cached sector `sector`. If the sector
    /// is not already cached, the sector is first read from the disk.
    ///
    /// The sector is marked dirty as a result of calling this method as it is
    /// presumed that the sector will be written to. If this is not intended,
    /// use `get()` instead.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get_mut(&mut self, sector: u64) -> io::Result<&mut [u8]> {
        // if self.cache.contains_key(&sector) {
        //     self.cache.get_mut(&sector).unwrap().dirty = true;
        //     Ok(self.cache.get_mut(&sector).unwrap().data.as_mut_slice())
        // } else {
        //     let factor = self.factor();
        //     let starting_physical_sector = self.virtual_to_physical(sector).expect("Error in virtual to physical");
        //     let ending_physical_sector = starting_physical_sector + factor;

        //     // virtual sector size should always be >= and a multiple of physical sector size
        //     let physical_sector_size = self.device.sector_size() as u64;
        //     let mut physical_sector_buf = vec![0u8; physical_sector_size as usize];
        //     let mut virtual_sector_buf = vec![0u8; self.partition.sector_size as usize];
        //     // let mut bytes_read = 0;
        //     // better way to do this with a single vector
        //     for physical_sector in starting_physical_sector..=ending_physical_sector {
        //         // let next_slice = virtual_sector_buf.as_mut_slice()[bytes_read..bytes_read + physical_sector_size];
        //         // bytes_read += self.device.read_sector(physical_sector, next_slice)? as u64;
        //         self.device.read_sector(physical_sector, physical_sector_buf.as_mut_slice());
        //         virtual_sector_buf.extend_from_slice(physical_sector_buf.as_slice());
        //     } 
        //     let new_cache_entry = CacheEntry {data: virtual_sector_buf, dirty: true};
        //     self.cache.insert(sector, new_cache_entry);
        //     Ok(self.cache.get_mut(&sector).unwrap().data.as_mut_slice())
        // }
        self.get(sector)?;
        let new_cache_entry = self.cache.get_mut(&sector).unwrap(); //unwrap is safe since .get() gurantees it is in the hashmap
        new_cache_entry.dirty = true;
        Ok(new_cache_entry.data.as_mut_slice())
    }

    /// Returns a reference to the cached sector `sector`. If the sector is not
    /// already cached, the sector is first read from the disk.
    ///
    /// # Errors
    ///
    /// Returns an error if there is an error reading the sector from the disk.
    pub fn get(&mut self, sector: u64) -> io::Result<&[u8]> {
        if self.cache.contains_key(&sector) {
            Ok(self.cache.get(&sector).unwrap().data.as_slice())
        } else {
            let factor = self.factor();
            let starting_physical_sector = self.virtual_to_physical(sector).expect("Error in virtual to physical");
            let ending_physical_sector = starting_physical_sector + factor;

            // virtual sector size should always be >= and a multiple of physical sector size
            let physical_sector_size = self.device.sector_size() as u64;
            // let mut physical_sector_buf = vec![0u8; physical_sector_size as usize];
            let mut virtual_sector_buf = Vec::new();
            // better way to do this with a single vector
            for physical_sector in starting_physical_sector..ending_physical_sector {
                // let next_slice = virtual_sector_buf.as_mut_slice()[bytes_read..bytes_read + physical_sector_size];
                // bytes_read += self.device.read_sector(physical_sector, next_slice)? as u64;
                self.device.read_all_sector(physical_sector, &mut virtual_sector_buf);
                // self.device.read_sector(physical_sector, physical_sector_buf.as_mut_slice());
                // virtual_sector_buf.extend_from_slice(physical_sector_buf.as_slice());
            } 
            let new_cache_entry = CacheEntry {data: virtual_sector_buf, dirty: false};
            self.cache.insert(sector, new_cache_entry);
            Ok(self.cache.get(&sector).unwrap().data.as_slice())
        }
    }

        // if let Some(found_cache_entry) = self.cache.get(&physical_sector) {
        //     Ok(found_cache_entry.data.as_slice())
        // } else {
        //     //what should the size of this vector be? partition sector size or physical sector size
        //     let mut vec_buf = vec![0u8; self.partition.sector_size as usize];
        //     self.device.read_sector(physical_sector, &mut vec_buf)?;
        //     let new_cache_entry = CacheEntry { data: vec_buf, dirty: false };
        //     self.cache.insert(physical_sector, new_cache_entry);
        //     Ok(vec_buf.as_slice())
        // }



            // let physical_sector = self.virtual_to_physical(sector).expect("Virtual sector number out of range");
            // match self.cache.entry(physical_sector) {
            //     Entry::Occupied(o) => {
            //         let _:() = o;
            //         Ok(o.get().data.as_slice())
            //     }, // vs & o.get().data
            //     Entry::Vacant(v) => {
            //         let mut vec_buf = vec![0u8; self.partition.sector_size as usize];
            //         self.device.read_sector(physical_sector, &mut vec_buf)?;
            //         let mut new_cache_entry = CacheEntry { data: vec_buf, dirty: false };
            //         v.insert(new_cache_entry);
            //         let a = vec_buf.pop();
            //         Ok(&[0;2])
            //     }
            // }
    // }
}

// FIXME: Implement `BlockDevice` for `CacheDevice`. The `read_sector` and
// `write_sector` methods should only read/write from/to cached sectors.
impl BlockDevice for CachedPartition {
    fn sector_size(&self) -> u64 {
        self.partition.sector_size
    }

    fn read_sector(&mut self, sector: u64, buf: &mut [u8]) -> io::Result<usize> {
        let cache_sector = self.get(sector)?;
        let num_bytes_to_read = core::cmp::max(cache_sector.len(), buf.len());
        buf[0..num_bytes_to_read].copy_from_slice(&cache_sector[0..num_bytes_to_read]);
        Ok(num_bytes_to_read)
    }

    fn write_sector(&mut self, sector: u64, buf: &[u8]) -> io::Result<usize> {
        let cache_sector = self.get_mut(sector)?;
        let num_bytes_to_read = core::cmp::max(cache_sector.len(), buf.len());
        cache_sector[0..num_bytes_to_read].copy_from_slice(&buf[0..num_bytes_to_read]);
        Ok(num_bytes_to_read)
    }
}

impl fmt::Debug for CachedPartition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CachedPartition")
            .field("device", &"<block device>")
            .field("cache", &self.cache)
            .finish()
    }
}


