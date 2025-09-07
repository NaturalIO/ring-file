use std::io::Write;
use crate::RingBuffer;
use std::path::Path;
use crossbeam_channel::*;
use std::thread;

enum Msg {
    Clear,
    Dump,
    Msg(String),
}

/// RingFile use a backend thread to maintain RingBuffer, which receive messages with unbounded channel,
/// to prevent lock contention affecting program execution.
/// When program hang or panic, you can call dump() to collect the logs into file.
pub struct RingFile {
    tx: Sender<Msg>,
    res: Receiver<std::io::Result<()>>,
    _th: thread::JoinHandle<()>,
}

struct RingFileBackend {
    file_path: Box<Path>,
    buffer: RingBuffer,
    rx: Receiver<Msg>,
    res: Sender<std::io::Result<()>>,
}

impl RingFileBackend {

    #[inline(always)]
    fn process(&mut self, msg: Msg) {
        match msg {
            Msg::Clear=>{
                self.buffer.clear();
            }
            Msg::Dump=>{
                let res = self.buffer.dump(self.file_path.as_ref());
                self.res.send(res).expect("send res");
            }
            Msg::Msg(line)=>{
                let _ = self.buffer.write_all(line.as_bytes());
            }
        }
    }

    fn run(&mut self) {
        loop {
            match self.rx.recv() {
                Ok(msg)=>{
                    self.process(msg);
                    while let Ok(msg) = self.rx.try_recv() {
                        self.process(msg);
                    }
                }
                Err(_)=>{
                    return;
                }
            }
        }
    }
}

impl RingFile {
    /// # Arguments:
    ///
    /// - buf_size: total buffer size
    ///
    /// - file_path: The target file to dump
    pub fn new(buf_size: i32, file_path: Box<Path>) -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        let (res_tx, res_rx) = crossbeam_channel::bounded(1);
        let mut backend = RingFileBackend {
            file_path,
            buffer: RingBuffer::new(buf_size),
            rx,
            res: res_tx,
        };
        let _th = thread::spawn(move || backend.run());
        Self{
            tx,
            _th,
            res: res_rx,
        }
    }

    /// Trigger dump to the disk.
    pub fn dump(&self) -> std::io::Result<()> {
        self.tx.send(Msg::Dump).expect("send");
        self.res.recv().unwrap()
    }

    #[inline(always)]
    pub fn write(&self, content: String) {
        self.tx.send(Msg::Msg(content)).expect("send");
    }

    /// Clear previous buffer
    pub fn clear(&self) {
        self.tx.send(Msg::Clear).expect("send");
    }
}
