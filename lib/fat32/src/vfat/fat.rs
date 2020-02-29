use crate::vfat::*;
use core::fmt;

use self::Status::*;

#[derive(Debug, PartialEq)]
pub enum Status {
    /// The FAT entry corresponds to an unused (free) cluster.
    Free,
    /// The FAT entry/cluster is reserved.
    Reserved,
    /// The FAT entry corresponds to a valid data cluster. The next cluster in
    /// the chain is `Cluster`.
    Data(Cluster),
    /// The FAT entry corresponds to a bad (disk failed) cluster.
    Bad,
    /// The FAT entry corresponds to a valid data cluster. The corresponding
    /// cluster is the last in its chain.
    Eoc(u32),
}

#[repr(C, packed)]
pub struct FatEntry(pub u32);

impl FatEntry {
    /// Returns the `Status` of the FAT entry `self`.
    pub fn status(&self) -> Status {
        let mask = !(0xF << 28);
        let masked_value = self.0 & mask;
        if masked_value == 0 {
            Status::Free
        } else if masked_value == 0x1 || (masked_value >= 0xFFFFFF0 && masked_value <= 0xFFFFFF6) {
            Status::Reserved
        } else if masked_value >= 0x2 && masked_value <= 0xFFFFFEF {
            Status::Data(Cluster::from(masked_value))
        } else if masked_value == 0xFFFFFF7 {
            Status::Bad
        } else { //no place for an ID FatEntry
            Status::Eoc(masked_value)
        }
    }
}

impl fmt::Debug for FatEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("FatEntry")
            .field("value", &{ self.0 })
            .field("status", &self.status())
            .finish()
    }
}
