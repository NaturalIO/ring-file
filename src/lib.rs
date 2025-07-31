//! # RingFile
//!
//! This crate provides a ring buffer like file, in order to store log content.
//! Integrated into [captain-log](https://docs.rs/captains-log) as `RingFile` log sink.
//!
//! The content keeps in memory when written, only the last part of buf_size is kept.
//! After write is done, you can dump the content to disk.

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

    /// Allocate a whole buffer specified by `buf_size`
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
