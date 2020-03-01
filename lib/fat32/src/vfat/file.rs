use alloc::string::String;

use shim::io::{self, SeekFrom};

use crate::traits;
use crate::vfat::{Cluster, Metadata, VFatHandle};

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    // FIXME: Fill me in.
    first_cluster: Cluster,
    cursor: usize
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.
impl<HANDLE: VFatHandle> traits::File for File<HANDLE> {
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!("Read-only file system")
    }
    fn size(&self) -> u64 {
        self.vfat.lock(|fat| fat.read_chain(self.first_cluster, &mut Vec::new())).expect("Error getting file size") as u64
    }
}
impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read_buf = Vec::new();
        // let _:() = read_buf;
        let read_bytes = self.vfat.lock(|fat| fat.read_chain(self.first_cluster, &mut read_buf)).expect("Error getting file size");
        let buf_bytes = core::cmp::min(buf.len(), (read_buf.len() - self.cursor) as usize);
        buf.copy_from_slice(&read_buf.as_slice()[self.cursor..self.cursor + buf_bytes]);

        Ok(buf_bytes)
    }
}
impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    
}

impl<HANDLE: VFatHandle> io::Seek for File<HANDLE> {
    /// Seek to offset `pos` in the file.
    ///
    /// A seek to the end of the file is allowed. A seek _beyond_ the end of the
    /// file returns an `InvalidInput` error.
    ///
    /// If the seek operation completes successfully, this method returns the
    /// new position from the start of the stream. That position can be used
    /// later with SeekFrom::Start.
    ///
    /// # Errors
    ///
    /// Seeking before the start of a file or beyond the end of the file results
    /// in an `InvalidInput` error.
    fn seek(&mut self, _pos: SeekFrom) -> io::Result<u64> {
        unimplemented!("File::seek()")
    }
}
