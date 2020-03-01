use alloc::string::String;
use alloc::vec::Vec;

use shim::const_assert_size;
use shim::ffi::OsStr;
use shim::io;
use shim::newioerr;

use crate::traits;
use crate::util::VecExt;
use crate::vfat::{Attributes, Date, Metadata, Time, Timestamp};
use crate::vfat::{Cluster, Entry, File, VFatHandle};

#[derive(Debug)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    // FIXME: Fill me in.
    start_cluster: Cluster,
    name: String,
    metadata: Metadata
}



pub struct EntryIter<HANDLE: VFatHandle> {
    vfat: HANDLE,
    dir_entries: Vec<VFatDirEntry>,
    index: usize
}

impl<HANDLE: VFatHandle> EntryIter<HANDLE> {
    //adds utf-16 chars to the vector to create the name and returns the number of chars added 
    fn build_long_name(vec: &mut Vec<u16>, lfn_dir_entry: VFatLfnDirEntry) -> u8 {
        let mut chars_added: u8 = 0;
        let end_of_name = false;
        for &utf16_char in &lfn_dir_entry.name_chars_1 {
            if utf16_char == 0x00 as u16 || utf16_char == 0xFF {
                return chars_added;
            } else {
                vec.push(utf16_char);
                chars_added += 1;
            }
        }
        for &utf16_char in &lfn_dir_entry.name_chars_2 {
            if utf16_char == 0x00 as u16 || utf16_char == 0xFF {
                return chars_added;
            } else {
                vec.push(utf16_char);
                chars_added += 1;
            }
        }
        for &utf16_char in &lfn_dir_entry.name_chars_3 {
            if utf16_char == 0x00 as u16 || utf16_char == 0xFF {
                return chars_added;
            } else {
                vec.push(utf16_char);
                chars_added += 1;
            }
        }
        chars_added
    }
    //adds name/extension chars to the string to create the name and returns the number of chars added 
    fn build_string(bytes: &[u8]) -> String {
        let byte_vec = Vec::new();
        for &byte in bytes {
            if byte == 0x0 || byte == 0x20 {
                break;
            } else {
                byte_vec.push(byte);
            }
        }
        String::from_utf8(byte_vec).expect("Error converting bytes to UTF-8 string")

    }
}

impl<HANDLE: VFatHandle> Iterator for EntryIter<HANDLE> {
    type Item = Entry<HANDLE>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.dir_entries.len() {
            return None;
        }
        let mut dir_entry = self.dir_entries[self.index];
        let mut unknown = unsafe { dir_entry.unknown };
        let mut long_name_bytes: Vec<u16> = Vec::new();
        while unknown.attributes.is_lfn() {
            let lfn_dir_entry = unsafe { dir_entry.long_filename };
            let bytes_added = EntryIter::build_long_name(&mut long_name_bytes, lfn_dir_entry);
            self.index += 1;
            dir_entry = self.dir_entries[self.index];
            unknown = unsafe { dir_entry.unknown };
        }
        let reg_dir_entry = unsafe { dir_entry.regular };
        // if the entry is a directory
        if reg_dir_entry.attributes.is_dir() {
            let mut final_name_string = String::new();
            if long_name_bytes.is_empty() {
                // this signifies the previous entry was the last entry or
                // this is a deleted/unused entry
                if reg_dir_entry.name[0] == 0x00 || reg_dir_entry.name[0] == 0xE5 {
                    return None;
                }
                let name_string = EntryIter::build_string(&reg_dir_entry.name);
                let extension_string = EntryIter::build_string(&reg_dir_entry.extension);
                final_name_string.push_str(&name_string);
                final_name_string.push_str(&extension_string);
            } else {
                let lfn_vec: Vec<u16> = core::char::decode_utf16(long_name_bytes.iter().cloned())
                    .map(|r| r.map_err(|e| e.unpaired_surrogate()))
                    .collect();
                let lfn_string = String::from_utf16(&lfn_vec).expect("error decoding utf-16");
                final_name_string.push_str(&lfn_string);
            }

            // get the metadata and starting cluster  

        } else {
            if long_name_bytes.is_empty() {
            
            } else {
    
            }
        }

        self.index += 1;
    }

}

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatRegularDirEntry {
    name: [u8; 8],
    extension: [u8; 3],
    attributes: Attributes,
    _reserved: u8,
    creation_time_tenths: u8,
    creation_timestamp: Timestamp,
    last_accessed_date: Date,
    high_cluster_number_bits: u16,
    last_modification_timestamp: Timestamp,
    low_cluster_number_bits: u16,
    file_size: u32
}

impl VFatRegularDirEntry {
    pub fn get_cluster(&self) -> Cluster {
        let base = !0u32;
        let cluster_num = (base & ((self.high_cluster_number_bits as u32) << 16u32)) & self.low_cluster_number_bits as u32; 
        Cluster::from(cluster_num)
    }
    pub fn get_metadata(&self) -> Metadata {
        Metadata {
            creation: self.creation_timestamp,
            last_access: Timestamp {
                time: self.last_modification_timestamp.time, //this is not correct
                date: self.last_accessed_date
            },

        }
    }
}

const_assert_size!(VFatRegularDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatLfnDirEntry {
    sequence_number: u8,
    name_chars_1: [u16; 5],
    attributes: Attributes,
    _type: u8,
    checksum: u8,
    name_chars_2: [u16; 6],
    lfn_zeros: u16,
    name_chars_3: [u16; 2]
}

const_assert_size!(VFatLfnDirEntry, 32);

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct VFatUnknownDirEntry {
    _irrelevant_1: [u8; 11],
    attributes: Attributes,
    _irrelevant_2: [u8; 20]
}

const_assert_size!(VFatUnknownDirEntry, 32);

pub union VFatDirEntry {
    unknown: VFatUnknownDirEntry,
    regular: VFatRegularDirEntry,
    long_filename: VFatLfnDirEntry,
}

impl<HANDLE: VFatHandle> Dir<HANDLE> {
    /// Finds the entry named `name` in `self` and returns it. Comparison is
    /// case-insensitive.
    ///
    /// # Errors
    ///
    /// If no entry with name `name` exists in `self`, an error of `NotFound` is
    /// returned.
    ///
    /// If `name` contains invalid UTF-8 characters, an error of `InvalidInput`
    /// is returned.
    pub fn find<P: AsRef<OsStr>>(&self, name: P) -> io::Result<Entry<HANDLE>> {
        unimplemented!("Dir::find()")
    }
}

impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    // FIXME: Implement `trait::Dir` for `Dir`.
}
