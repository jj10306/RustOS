use core::fmt;
use shim::const_assert_size;
use shim::io;

use crate::traits::BlockDevice;

#[repr(C)]
#[derive(Copy, Clone)]
pub struct CHS {
    head_sector_cylinder: [u8; 3]
}

impl fmt::Debug for CHS {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("CHS")
            .field("head_sector_cylinder", &self.head_sector_cylinder)
            .finish()
    }
}

const_assert_size!(CHS, 3);

#[repr(C, packed)]
pub struct PartitionEntry { 
    bootable_indicator: u8,
    starting_chs: CHS,
    partition_type: u8,
    ending_chs: CHS,
    pub relative_sector: u32,
    pub total_sectors: u32

}

impl fmt::Debug for PartitionEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("PartitionEntry")
            .field("bootable_indicator", &self.bootable_indicator)
            .field("starting_chs", &self.starting_chs)
            .field("partition_type", &self.partition_type)
            .field("ending_chs", &self.ending_chs)
            .field("relative_sector", &self.relative_sector)
            .field("total_sectors", &self.total_sectors)
            .finish()
    }
}

const_assert_size!(PartitionEntry, 16);

/// The master boot record (MBR).
#[repr(C, packed)]
pub struct MasterBootRecord {
    bootstrap: [u8; 436],
    id: [u8; 10],
    pub partition_table_entry_1: PartitionEntry,
    pub partition_table_entry_2: PartitionEntry,
    pub partition_table_entry_3: PartitionEntry,
    pub partition_table_entry_4: PartitionEntry,
    valid_bootsector: u16
}

impl fmt::Debug for MasterBootRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MasterBootRecord")
            //why does this work? &self.bootstrap doesn't work b/c trairts are not implemented on arrays of size > 32
            .field("bootstrap", &&self.bootstrap[..])
            .field("id", &self.id)
            .field("partition_table_entry_1", &self.partition_table_entry_1)
            .field("partition_table_entry_2", &self.partition_table_entry_2)
            .field("partition_table_entry_3", &self.partition_table_entry_3)
            .field("partition_table_entry_4", &self.partition_table_entry_4)
            .field("valid_bootsector", &self.valid_bootsector)
            .finish()
    }
}

const_assert_size!(MasterBootRecord, 512);

#[derive(Debug)]
pub enum Error {
    /// There was an I/O error while reading the MBR.
    Io(io::Error),
    /// Partiion `.0` (0-indexed) contains an invalid or unknown boot indicator.
    UnknownBootIndicator(u8),
    /// The MBR magic signature was invalid.
    BadSignature,
}

impl MasterBootRecord {
    /// Reads and returns the master boot record (MBR) from `device`.
    ///
    /// # Errors
    ///
    /// Returns `BadSignature` if the MBR contains an invalid magic signature.
    /// Returns `UnknownBootIndicator(n)` if partition `n` contains an invalid
    /// boot indicator. Returns `Io(err)` if the I/O error `err` occured while
    /// reading the MBR.
   pub fn from<T: BlockDevice>(mut device: T) -> Result<MasterBootRecord, Error> {
        let buf = &mut [0u8; 512];
        if let Err(e) = device.read_sector(0, buf) {
            return Err(Error::Io(e));
        }
        // let mbr = unsafe{ core::mem::transmute::<[u8; 512], MasterBootRecord>(*buf) };
        // let dir_entries: Vec<VFatDirEntry> = unsafe{ entries_as_bytes_buf.cast() };
        let mbr = unsafe{ core::mem::transmute::<[u8; 512], MasterBootRecord>(*buf) };

        if mbr.valid_bootsector != 0xAA55 {
            return Err(Error::BadSignature);
        } 
        
        if let Some(n) = MasterBootRecord::has_valid_bootable_indicators(&mbr) {
            Err(Error::UnknownBootIndicator(n))
        } else {
            Ok(mbr)
        }
    }
    fn has_valid_bootable_indicators(mbr: &MasterBootRecord) -> Option<u8> {
        if mbr.partition_table_entry_1.bootable_indicator != 0 && mbr.partition_table_entry_1.bootable_indicator != 0x80 {
            Some(0)
        } else if mbr.partition_table_entry_2.bootable_indicator != 0 && mbr.partition_table_entry_2.bootable_indicator != 0x80 {
            Some(1)
        } else if mbr.partition_table_entry_3.bootable_indicator != 0 && mbr.partition_table_entry_3.bootable_indicator != 0x80 {
            Some(2)
        } else if mbr.partition_table_entry_4.bootable_indicator != 0 && mbr.partition_table_entry_4.bootable_indicator != 0x80 {
            Some(3)
        } else {
            None
        }
    }
}
