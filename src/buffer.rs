use io_buffer::{safe_copy, Buffer};
use std::fs::*;
use std::io::{Result, Write};
use std::path::Path;

/// The content is kept in memory when written, when offset rewinds, new content will overwrite old content,
/// So that memory consumption is limited to buf_size.
/// Once deadlock encountered and process hangs, no more message will be written,
/// you can safely dump the content to disk.
///
/// # Example:
///
/// ```rust
/// use ring_file::RingBuffer;
/// use std::io::Write;
/// let mut file = RingBuffer::new(512*1024*1024);
/// file.write_all(b"log message").expect("write ok");
/// file.dump("/tmp/ringfile.store").expect("dump ok");
/// ```
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
        Self { end: 0, inner, full: false }
    }

    /// Will create a truncated file and write all data from mem to disk.
    pub fn dump<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let mut file =
            OpenOptions::new().write(true).create(true).truncate(true).open(file_path.as_ref())?;
        if self.full {
            file.write_all(&self.inner[self.end..])?;
            return file.write_all(&self.inner[0..self.end]);
        } else {
            return file.write_all(&self.inner[0..self.end]);
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
