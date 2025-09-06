use std::cell::UnsafeCell;
use std::fs::OpenOptions;
use std::io::{stdout, Write};
use std::mem::transmute;
use std::collections::LinkedList;
use std::path::Path;
use std::sync::{
    atomic::{AtomicU8, AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use thread_local::ThreadLocal;

#[derive(Debug, PartialEq)]
#[repr(u8)]
enum BufferState {
    Unlock,
    Lock,
    Dump,
}

struct LocalBufferMut {
    logs: LinkedList<(i64, String)>,
    cur_size: usize,
    size_limit: usize,
}

impl LocalBufferMut {

    #[inline(always)]
    fn new(buf_size: usize) -> Self {
        Self{
            logs: LinkedList::new(),
            cur_size: 0,
            size_limit: buf_size,
        }
    }

    #[inline(always)]
    fn push(&mut self, ts: i64, content: String) {
        self.cur_size += content.len();
        self.logs.push_back((ts, content));
        while self.cur_size > self.size_limit && self.logs.len() > 1 {
            if let Some((_, old)) = self.logs.pop_front() {
                self.cur_size -= old.len();
            } else {
                unreachable!();
            }
        }
    }
}

struct LocalBuffer {
    inner: UnsafeCell<LocalBufferMut>,
    locked: AtomicU8,
}

unsafe impl Send for LocalBuffer {}
unsafe impl Sync for LocalBuffer {}

impl LocalBuffer {
    #[inline(always)]
    fn new(buf_size: usize) -> Arc<Self> {
        Arc::new(Self {
            inner: UnsafeCell::new(LocalBufferMut::new(buf_size)),
            locked: AtomicU8::new(BufferState::Unlock as u8),
        })
    }

    #[inline(always)]
    fn write(&self, ts: i64, buf: String) {
        loop {
            match self.try_lock(BufferState::Unlock, BufferState::Lock) {
                Ok(_) => {
                    let inner = self.get_inner_mut();
                    inner.push(ts, buf);
                    self.locked.store(BufferState::Unlock as u8, Ordering::Release);
                    return;
                }
                Err(s) => {
                    if s == BufferState::Dump as u8 {
                        std::thread::sleep(Duration::from_millis(100));
                    } else {
                        unreachable!();
                    }
                }
            }
        }
    }

    #[inline]
    fn collect(&self, all: &mut Vec<(i64, String)>) {
        loop {
            match self.try_lock(BufferState::Unlock, BufferState::Dump) {
                Ok(_) => {
                    {
                        let inner = self.get_inner();
                        for (ts, line) in inner.logs.iter() {
                            all.push((*ts, line.clone()));
                        }
                    }
                    self.locked.store(BufferState::Unlock as u8, Ordering::Release);
                    return;
                }
                Err(s) => {
                    if s == BufferState::Lock as u8 {
                        std::hint::spin_loop();
                    } else {
                        return;
                    }
                }
            }
        }
    }

    #[inline(always)]
    fn try_lock(&self, state: BufferState, target: BufferState) -> Result<(), u8> {
        match self.locked.compare_exchange(
            state as u8,
            target as u8,
            Ordering::Acquire,
            Ordering::Relaxed,
        ) {
            Ok(_) => Ok(()),
            Err(s) => Err(s),
        }
    }

    #[inline(always)]
    fn get_inner(&self) -> &LocalBufferMut {
        unsafe { transmute(self.inner.get()) }
    }

    #[inline(always)]
    fn get_inner_mut(&self) -> &mut LocalBufferMut {
        unsafe { transmute(self.inner.get()) }
    }
}

/// RingFile keeps [RingBuffer] within thread local, to prevent lock contention affecting program
/// execution.
/// When program hang or panic, you can call dump() to collect the logs into file or stdout.
pub struct RingFile {
    file_path: Option<Box<Path>>,
    buf_size: usize,
    buffers: ThreadLocal<Arc<LocalBuffer>>,
    count: AtomicUsize,
}

impl RingFile {
    /// # Arguments:
    ///
    /// - buf_size: buffer size per thread
    ///
    /// - file_path: If contains a path, the target is a file, otherwise will write to stdout.
    pub fn new(buf_size: usize, file_path: Option<Box<Path>>) -> Self {
        Self {
            file_path,
            buf_size,
            count: AtomicUsize::new(0),
            buffers: ThreadLocal::with_capacity(32),
        }
    }

    /// collect all the buffers, sort by timestamp and dump to disk or stdout.
    pub fn dump(&self) -> std::io::Result<()> {
        let mut all: Vec<(i64, String)>;
        {
            let mut est = self.count.load(Ordering::Relaxed) * self.buf_size / 100;
            if est < 100 {
                est = 100;
            }
            all = Vec::with_capacity(2 * est);
            for buf in self.buffers.iter() {
                buf.collect(&mut all);
            }
        }
        all.sort_by(|a, b| a.0.cmp(&b.0));
        macro_rules! dump_all {
            ($f: expr) => {
                for (_, line) in all {
                    if let Err(e) = $f.write_all(line.as_bytes()) {
                        println!("RingFile: dump error {:?}", e);
                        return Err(e);
                    }
                }
                $f.flush()?;
            };
        }
        if let Some(path) = self.file_path.as_ref() {
            match OpenOptions::new().write(true).create(true).truncate(true).open(path) {
                Ok(mut f) => {
                    dump_all!(f);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            let mut f = stdout().lock();
            dump_all!(f);
        }
        Ok(())
    }

    #[inline(always)]
    pub fn write(&self, ts: i64, content: String) {
        let buf = self.buffers.get_or(|| {
            let _ = self.count.fetch_add(1, Ordering::Relaxed);
            LocalBuffer::new(self.buf_size)
        });
        buf.write(ts, content);
    }

    pub fn clear(&mut self) {
        self.buffers.clear();
    }
}
