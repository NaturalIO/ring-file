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
//! use ring_file::RingBuffer;
//! use std::io::Write;
//! let mut file = RingBuffer::new(512*1024*1024);
//! file.write_all(b"log message").expect("write ok");
//! file.dump("/tmp/ringfile.store").expect("dump ok");
//! ```

use std::path::Path;
use io_buffer::{Buffer, safe_copy};
use std::io::{Result, Write};
use std::fs::*;


pub struct RingBuffer {
    end: usize,
    full: bool,
    inner: Buffer,
}

impl RingBuffer {

    /// Allocate a whole buffer specified by `buf_size`, size can not exceed 2GB.
    pub fn new(buf_size: i32) -> Self {
        assert!(buf_size > 0);
        let inner = Buffer::alloc(buf_size).expect("alloc");
        Self{
            end: 0,
            inner,
            full: false,
        }
    }

    /// Will create a truncated file and write all data from mem to disk.
    pub fn dump<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let mut file = OpenOptions::new().write(true).create(true).truncate(true).open(file_path.as_ref())?;
        if self.full {
            file.write_all(&self.inner[self.end..])?;
            return file.write_all(&self.inner[0..self.end]);
        } else {
            return file.write_all(&self.inner[0..self.end]);
        }
    }

    /// Split the content by lines and append to param `arr`
    pub fn collect_into(&self, mut arr: Vec<Vec<u8>>) {
        let mut _line: Option<Vec<u8>> = None;
        if self.full {
            for line in self.inner[self.end..0].split(|b| *b == b'\n') {
                if let Some(line_freeze) = _line.replace(line.to_vec()) {
                    arr.push(line_freeze);
                }
            }
            for line in self.inner[0..self.end].split(|b| *b == b'\n') {
                if let Some(mut pre_line) = _line.take() {
                    if pre_line[pre_line.len()-1] == b'\n' {
                        arr.push(pre_line);
                        arr.push(line.to_vec());
                    } else {
                        pre_line.append(&mut line.to_vec());
                        arr.push(pre_line);
                    }
                } else {
                    arr.push(line.to_vec());
                }
            }
        } else {
            for line in self.inner[0..self.end].split(|b| *b == b'\n') {
                arr.push(line.to_vec());
            }
        }
    }
}

impl std::io::Write for RingBuffer {

    /// Write will abort when reaching the boundary of buffer, rewind the offset to 0 and return the bytes written.
    /// You can use Write::write_all() provided by the trait to cover the rewinding logic.
    #[inline]
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let bound = self.inner.capacity();
        let l = buf.len();
        if self.end + l >= bound {
            self.full = true;
            let l1 = safe_copy(&mut self.inner[self.end..], &buf);
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
