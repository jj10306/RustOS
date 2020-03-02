use core::fmt;

use alloc::string::String;

use crate::traits;

/// A date as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Date(u16);

impl Date {
    pub fn new(t: u16) -> Date {
        Date(t)
    }
}

/// Time as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Time(u16);

impl Time {
    pub fn new(t: u16) -> Time {
        Time(t)
    }
}
/// File attributes as represented in FAT32 on-disk structures.
#[repr(C, packed)]
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub struct Attributes(u8);
impl Attributes {
    pub fn is_lfn(&self) -> bool {
        self.0 == 0x0F
    }
    pub fn is_dir(&self) -> bool {
        self.0 == 0x10
    }
}
/// A structure containing a date and time.
#[derive(Default, Copy, Clone, Debug, PartialEq, Eq)]
pub struct Timestamp {
    pub time: Time,
    pub date: Date
}
impl Timestamp {
    pub fn new(time: Time, date: Date) -> Timestamp {
        Timestamp {
            time,
            date  
        }
    }
}

/// Metadata for a directory entry.
#[derive(Default, Debug, Clone)]
pub struct Metadata {
    pub creation: Timestamp,
    pub last_access: Timestamp,
    pub modified: Timestamp,
    pub attributes: Attributes
}

impl Metadata {
    pub fn new(creation: Timestamp, last_access: Timestamp, modified: Timestamp, attributes: Attributes) -> Metadata {
        Metadata {
            creation,
            last_access,
            modified,
            attributes
        }
    }
    pub fn new_root_meta() -> Metadata {
        Metadata {
            creation: Timestamp::new(Time::new(0), Date::new(0)),
            last_access: Timestamp::new(Time::new(0), Date::new(0)),
            modified: Timestamp::new(Time::new(0), Date::new(0)),
            attributes: Attributes(0x10)
        }
    }
}

// FIXME: Implement `traits::Timestamp` for `Timestamp`.
impl traits::Timestamp for Timestamp {
    /// The calendar year.
    ///
    /// The year is not offset. 2009 is 2009.
    fn year(&self) -> usize {
        let mask = 0b1111111 << 9;
        (self.date.0 & mask) as usize
    }

    /// The calendar month, starting at 1 for January. Always in range [1, 12].
    ///
    /// January is 1, Feburary is 2, ..., December is 12.
    fn month(&self) -> u8 {
        let mask = 0b1111 << 5;
        (self.date.0 & mask) as u8
    }

    /// The calendar day, starting at 1. Always in range [1, 31].
    fn day(&self) -> u8 {
        let mask = 0b11111 << 0;
        (self.date.0 & mask) as u8
    }

    /// The 24-hour hour. Always in range [0, 24).
    fn hour(&self) -> u8 {
        let mask = 0b11111 << 11;
        (self.time.0 & mask) as u8
    }

    /// The minute. Always in range [0, 60).
    fn minute(&self) -> u8 {
        let mask = 0b111111 << 5;
        (self.time.0 & mask) as u8
    }

    /// The second. Always in range [0, 60).
    fn second(&self) -> u8 {
        let mask = 0b11111 << 0;
        ((self.time.0 & mask) * 2) as u8
    }
}


// FIXME: Implement `traits::Metadata` for `Metadata`.

impl traits::Metadata for Metadata {
    /// Type corresponding to a point in time.
    type Timestamp = Timestamp;

    /// Whether the associated entry is read only.
    fn read_only(&self) -> bool {
        self.attributes.0 == 0x01
    }
    /// Whether the entry should be "hidden" from directory traversals.
    fn hidden(&self) -> bool {
        self.attributes.0 == 0x02
    }
    /// The timestamp when the entry was created.
    fn created(&self) -> Self::Timestamp {
        self.creation
    }
    /// The timestamp for the entry's last access.
    fn accessed(&self) -> Self::Timestamp {
        self.last_access
    }
    /// The timestamp for the entry's last modification.
    fn modified(&self) -> Self::Timestamp {
        self.modified
    }
}

// FIXME: Implement `fmt::Display` (to your liking) for `Metadata`.
impl fmt::Display for Metadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("MetaData")
            .field("Created", &self.creation)
            .field("Last Accessed", &self.last_access)
            .field("Last Modified", &self.modified)
            .field("Attributes", &self.attributes)
            .finish()
    }
}

