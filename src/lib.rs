//! # RingFile
//!
//! The purpose of this tool is to help debug deadlock problems, that only occur under
//! high-performance scenarios. Because writing log to disk will slow down execution,
//! which makes deadlock hidden without racing conditions met.
//!
//! This crate provides a ringbuffer like file, in order to store byte content.
//! Already integrated into [captain-log](https://docs.rs/captains-log) as `LogRingFile` sink.
//!
//! The content is kept in memory when written, when offset rewinds, new content will overwrite old content,
//! So that memory consumption is limited to buf_size.
//! Once deadlock encountered and process hangs, no more message will be written,
//! you can safely dump the content to disk.
//!
//! # Example:
//!
//! ```rust
//! use ring_file::RingFile;
//! let mut file = RingFile::new(512*1024*1024, "/tmp/ringfile.store");
//! file.write_all("log message").expect("write ok");
//! file.dump().expect("dump ok");
//! ```

use std::path::PathBuf;
use io_buffer::{Buffer, safe_copy};
use std::io::{Result, Write};
use std::fs::*;

pub struct RingFile {
    end: usize,
    inner: Buffer,
    full: bool,
    file_path: PathBuf,
}

impl RingFile {

    /// Allocate a whole buffer specified by `buf_size`, size can not exceed 2GB.
    pub fn new<P: Into<PathBuf>>(buf_size: i32, file_path: P) -> Self {
        assert!(buf_size > 0);
        let inner = Buffer::alloc(buf_size).expect("alloc");
        Self{
            end: 0,
            inner,
            full: false,
            file_path: file_path.into(),
        }
    }

    /// Will create a truncated file and write all data from mem to disk.
    pub fn dump(&self) -> Result<()> {
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(&self.file_path)?;
        if self.full {
            file.write_all(&self.inner[self.end..])?;
            return file.write_all(&self.inner[0..self.end]);
        } else {
            return file.write_all(&self.inner[0..self.end]);
        }
    }

}

impl std::io::Write for RingFile {

    /// Write will abort when reaching the boundary of buffer, rewind the offset to 0 and return the bytes written.
    /// You can use Write::write_all() provided by the trait to cover the rewinding logic.
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let bound = self.inner.capacity();
        let l = buf.len();
        if self.end + l >= bound {
            let l1 = safe_copy(&mut self.inner[self.end..], &buf);
            self.full = true;
            self.end = 0;
            return Ok(l1);
        } else {
            safe_copy(&mut self.inner[self.end..], buf);
            self.end += l;
            return Ok(l);
        }
    }

    #[inline]
    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}
