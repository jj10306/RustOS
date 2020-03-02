use alloc::string::String;

use shim::io::{self, SeekFrom};
use shim::ioerr;

use crate::traits;
use crate::vfat::{Cluster, Metadata, VFatHandle};

#[derive(Debug)]
pub struct File<HANDLE: VFatHandle> {
    pub vfat: HANDLE,
    // FIXME: Fill me in.
    pub start_cluster: Cluster,
    pub name: String,
    pub cursor: usize,
    pub size: usize,
    pub metadata: Metadata
}

// FIXME: Implement `traits::File` (and its supertraits) for `File`.
impl<HANDLE: VFatHandle> traits::File for File<HANDLE> {
    fn sync(&mut self) -> io::Result<()> {
        unimplemented!("Read-only file system")
    }
    fn size(&self) -> u64 {
        // self.vfat.lock(|fat| fat.read_chain(self.first_cluster, &mut Vec::new())).expect("Error getting file size") as u64
        self.size as u64
    }
}
impl<HANDLE: VFatHandle> io::Read for File<HANDLE> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut read_buf = Vec::new();
        // let _:() = read_buf;
        let read_bytes = self.vfat.lock(|fat| fat.read_chain(self.start_cluster, &mut read_buf)).expect("Error getting file size");
        let buf_bytes = core::cmp::min(buf.len(), (read_buf.len() - self.cursor) as usize);
        buf.copy_from_slice(&read_buf.as_slice()[self.cursor..self.cursor + buf_bytes]);

        Ok(buf_bytes)
    }
}
impl<HANDLE: VFatHandle> io::Write for File<HANDLE> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        unimplemented!("read-only, for now (;");
    }
    fn flush(&mut self) -> io::Result<()> {
        unimplemented!("read-only, for now (;");
    }
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
        let file_size = self.size;
        match _pos {
            SeekFrom::Start(seek) => {
                let final_cursor = self.cursor as u64 + seek;
                if final_cursor < 0 || final_cursor >= file_size as u64 {
                    ioerr!(InvalidInput, "Seeked before the start or beyond the end of the file")
                } else {
                    Ok(final_cursor)
                }
            },
            SeekFrom::End(seek) => {
                let final_cursor = self.cursor as i64 + seek;
                if final_cursor < 0 || final_cursor >= file_size as i64 {
                    ioerr!(InvalidInput, "Seeked before the start or beyond the end of the file")
                } else {
                    Ok(final_cursor as u64)
                }
            },
            SeekFrom::Current(seek) => {
                let final_cursor = self.cursor as i64 + seek;
                if final_cursor < 0 || final_cursor >= file_size as i64 {
                    ioerr!(InvalidInput, "Seeked before the start or beyond the end of the file")
                } else {
                    Ok(final_cursor as u64)
                }
            }
        }
    }
}
