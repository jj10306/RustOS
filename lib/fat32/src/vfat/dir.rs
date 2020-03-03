use alloc::string::String;
use alloc::vec::Vec;

use shim::const_assert_size;
use shim::ffi::OsStr;
use shim::io;
use shim::newioerr;
use shim::ioerr;

use crate::traits;
use crate::util::VecExt;
use crate::vfat::{Attributes, Date, Metadata, Time, Timestamp};
use crate::vfat::{Cluster, Entry, File, VFatHandle};

#[derive(Debug)]
pub struct Dir<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    // FIXME: Fill me in.
    start_cluster: Cluster,
    pub name: String,
    pub metadata: Metadata
}

impl<HANDLE: VFatHandle> Dir<HANDLE> {
    pub fn new(vfat: HANDLE, start_cluster: Cluster, name: String) -> Dir<HANDLE> {
        Dir {
            vfat,
            start_cluster,
            name,
            metadata: Metadata::new_root_meta()
        }
    }
}

pub struct EntryIter<HANDLE: VFatHandle> {
    vfat: HANDLE,
    dir_entries: Vec<VFatDirEntry>,
    index: usize
}

impl<HANDLE: VFatHandle> EntryIter<HANDLE> {
    //adds utf-16 chars to the vector to create the name and returns the number of chars added 
    fn build_long_name_bytes(&self, vec: &mut Vec<u16>, lfn_dir_entry: VFatLfnDirEntry) -> u8 {
        let mut chars_added: u8 = 0;
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

    // fn build_long_name_string(long_name_bytes: Vec<u16>) -> String {
    //     String::from_utf16_lossy(&long_name_bytes)
    // }

    //adds name/extension chars to the string to create the name and returns the number of chars added 
    fn build_string(&self, bytes: &[u8]) -> String {
        let mut byte_vec = Vec::new();
        for &byte in bytes {
            if byte == 0x0 || byte == 0x20 {
                break;
            } else {
                byte_vec.push(byte);
            }
        }
        // let s = String::from_utf8_lossy(byte_vec.as_slice());
        // s.to_string()
        String::from_utf8(byte_vec).unwrap()

    }
}

impl<HANDLE: VFatHandle> Iterator for EntryIter<HANDLE> {
    type Item = Entry<HANDLE>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.dir_entries.len() {
            return None;
        }
        let mut dir_entry = &self.dir_entries[self.index];
        let mut unknown = unsafe { dir_entry.unknown };
        let mut lfn_tuples: Vec<(u8, String)> = Vec::new();
        let mut capture = false;
        while unknown.attributes.is_lfn() {
            let lfn_dir_entry = unsafe { dir_entry.long_filename };
            if !lfn_dir_entry.is_deleted() { 
                //TODO check if it last entry 0x00
                if lfn_dir_entry.is_start() { capture = true; }
                else if lfn_dir_entry.is_end() { capture = false }

                if capture {
                    let mut long_name_bytes: Vec<u16> = Vec::new();
                    let bytes_added: u8 = self.build_long_name_bytes(&mut long_name_bytes, lfn_dir_entry);
                    let long_name_string = String::from_utf16_lossy(&long_name_bytes);
                    lfn_tuples.push((lfn_dir_entry.sequence_number, long_name_string));
                }
             }

            self.index += 1;
            dir_entry = &self.dir_entries[self.index];
            unknown = unsafe { dir_entry.unknown };
        }
        let reg_dir_entry = unsafe { dir_entry.regular };

        //these are the fields that File and Dir share 
        let metadata;
        let start_cluster;
        let size;
        let mut name = String::new();

        if lfn_tuples.is_empty() {
            // this signifies the previous entry was the last entry or
            // this is a deleted/unused entry
            if reg_dir_entry.name[0] == 0x00  {
                return None;
            }
            if reg_dir_entry.name[0] == 0xE5 {
                self.index += 1;
                return self.next();
            }
            let name_string = self.build_string(&reg_dir_entry.name);
            let extension_string = self.build_string(&reg_dir_entry.extension);
            name.push_str(&name_string);
            //only add if extension exists
            if reg_dir_entry.extension[0] != 0x00 && reg_dir_entry.extension[0] != 0x20 {
                name.push('.');
            }
            name.push_str(&extension_string);
        } else {
            // transform tuple vec into the file name string
            lfn_tuples.sort_by_key(|k| k.0);
            let strings: Vec<String> = lfn_tuples.into_iter().map(|t| t.1).collect();
            name = strings.join("")
            //name.push_str(&lfn_string);
        }

        // get the metadata and starting cluster  
        metadata = reg_dir_entry.get_metadata();
        start_cluster = reg_dir_entry.get_cluster();
        size = reg_dir_entry.get_size() as usize;

        // be sure to increment the iterator's pointer
        self.index += 1;
        if reg_dir_entry.attributes.is_dir() {
            let dir = Dir {
                vfat: self.vfat.clone(),
                start_cluster,
                name,
                metadata
            };
            Some(Entry::Dir(dir))
        } else {
            let file = File {
                vfat: self.vfat.clone(),
                start_cluster,
                name,
                cursor: 0,
                size,
                metadata
            };
            Some(Entry::File(file))
        }
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
    creation_time: Time,
    creation_date: Date,
    last_accessed_date: Date,
    high_cluster_number_bits: u16,
    last_modification_time: Time,
    last_modification_date: Date,
    low_cluster_number_bits: u16,
    file_size: u32
}

impl VFatRegularDirEntry {
    pub fn get_cluster(&self) -> Cluster {
        let base = !0u32;
        let cluster_num = (base & ((self.high_cluster_number_bits as u32) << 16u32)) + self.low_cluster_number_bits as u32; 
        Cluster::from(cluster_num)
    }
    pub fn get_metadata(&self) -> Metadata {
        let creation_timestamp = Timestamp::new(self.creation_time, self.creation_date);
        let accessed_timestamp = Timestamp::new(Time::new(0), self.last_accessed_date);
        let modification_timestamp = Timestamp::new(self.last_modification_time, self.last_modification_date);
        Metadata::new(creation_timestamp, accessed_timestamp, modification_timestamp, self.attributes)
    }
    pub fn get_size(&self) -> u32 {
        self.file_size
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
impl VFatLfnDirEntry {
    pub fn is_start(&self) -> bool {
        let shifty = 0x1 << 5;
        self.sequence_number & shifty == 0
    }
    pub fn is_end(&self) -> bool {
        let shifty = 0x1 << 6;
        self.sequence_number & shifty == 1
    }
    pub fn is_deleted(&self) -> bool {
        self.sequence_number == 0xE5
    }
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

        let entries = traits::Dir::entries(self)?;
        let name_as_str = match name.as_ref().to_str() {
            Some(name) => name,
            None => {return ioerr!(InvalidInput, "Invalid UTF-8 chars");}
        };
        for entry in entries {
            match &entry {
                Entry::File(file) => {
                    if file.name.eq_ignore_ascii_case(name_as_str) {
                        return Ok(entry);
                    }
                },
                Entry::Dir(dir) => {
                    if dir.name.eq_ignore_ascii_case(name_as_str) {
                        return Ok(entry);
                    }
                }
            }
        }
        ioerr!(NotFound, "No entry with name in directory")
    }
}

impl<HANDLE: VFatHandle> traits::Dir for Dir<HANDLE> {
    // FIXME: Implement `trait::Dir` for `Dir`.
    type Entry = Entry<HANDLE>;

    type Iter = EntryIter<HANDLE>;

    fn entries(&self) -> io::Result<Self::Iter> {
        let mut entries_as_bytes_buf = Vec::new();
        let start_cluster = self.start_cluster;
        self.vfat.lock(|fat| fat.read_chain(start_cluster, &mut entries_as_bytes_buf));
        let dir_entries: Vec<VFatDirEntry> = unsafe{ entries_as_bytes_buf.cast() };
        Ok(EntryIter {
            vfat: self.vfat.clone(),
            dir_entries,
            index: 0
        })
    }
}
