//! # RingFile
//!
//! The purpose of this tool is to help debug deadlock problems, that only occur under
//! high-performance scenarios. Because writing log to disk will slow down execution,
//! which makes deadlock hidden without racing conditions met.
//!
//! This crate provides two abstraction:
//!
//! [RingBuffer]: to store byte content in memory when written.
//! when offset rewinds, new content will overwrite old content,
//! so that memory consumption is limited to buf_size.
//!
//! [RingFile]: to record log content in memory for multi-threaded program. Act as an observer to
//! analyze concurrency problem. It maintain thread local buffer to avoid lock contention.
//! Already integrated into [captain-log](https://docs.rs/captains-log) as `LogRingFile` sink.

mod buffer;
pub use buffer::RingBuffer;
mod threads;
pub use threads::RingFile;
