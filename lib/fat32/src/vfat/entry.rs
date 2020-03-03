use crate::traits;
use crate::vfat::{Dir, File, Metadata, VFatHandle};
use core::fmt;

// You can change this definition if you want
#[derive(Debug)]
pub enum Entry<HANDLE: VFatHandle> {
    File(File<HANDLE>),
    Dir(Dir<HANDLE>)
}

// TODO: Implement any useful helper methods on `Entry`.

impl<HANDLE: VFatHandle> traits::Entry for Entry<HANDLE> {
    // FIXME: Implement `traits::Entry` for `Entry`.
    type File = File<HANDLE>;
    type Dir = Dir<HANDLE>;
    type Metadata = Metadata;

    /// The name of the file or directory corresponding to this entry.
    fn name(&self) -> &str {
        match self {
            Entry::File(file) => file.name.as_str(),
            Entry::Dir(dir) => dir.name.as_str() 
        }
    }

    /// The metadata associated with the entry.
    fn metadata(&self) -> &Self::Metadata {
        match self {
            Entry::File(file) => {
                return &file.metadata;},
            Entry::Dir(dir) => &dir.metadata 
        }
    }
    /// If `self` is a file, returns `Some` of a reference to the file.
    /// Otherwise returns `None`.
    fn as_file(&self) -> Option<&File<HANDLE>> {
        match self {
            Entry::File(file) => Some(&file),
            Entry::Dir(_) => None
        }
    }

    /// If `self` is a directory, returns `Some` of a reference to the
    /// directory. Otherwise returns `None`.
    fn as_dir(&self) -> Option<&Dir<HANDLE>> {
        match self {
            Entry::File(_) => None,
            Entry::Dir(dir) => Some(&dir) 
        }
    }

    /// If `self` is a file, returns `Some` of the file. Otherwise returns
    /// `None`.
    fn into_file(self) -> Option<File<HANDLE>> {
        match self {
            Entry::File(file) => Some(file),
            Entry::Dir(dir) => None
        }
    }

    /// If `self` is a directory, returns `Some` of the directory. Otherwise
    /// returns `None`.
    fn into_dir(self) -> Option<Dir<HANDLE>> {
        match self {
            Entry::File(file) => None,
            Entry::Dir(dir) => Some(dir) 
        }
    }

    /// Returns `true` if this entry is a file or `false` otherwise.
    fn is_file(&self) -> bool {
        self.as_file().is_some()
    }

    /// Returns `true` if this entry is a directory or `false` otherwise.
    fn is_dir(&self) -> bool {
        self.as_dir().is_some()
    }
}
