# RingFile

crates.io: [![crates.io][cratesio-image]][cratesio]
docs.rs: [![docs.rs][docsrs-image]][docsrs]

[cratesio-image]: https://img.shields.io/crates/v/ring-file.svg
[cratesio]: https://crates.io/crates/ring-file
[docsrs-image]: https://docs.rs/ring-file/badge.svg
[docsrs]: https://docs.rs/ring-file

The purpose of this tool is to help debug deadlock problems, that only occur under
high-performance scenarios. Because writing log to disk will slow down execution,
which makes deadlock hidden without racing conditions met.

This crate provides two abstraction:

`RingBuffer`: to store byte content in memory when written.
when offset rewinds, new content will overwrite old content,
so that memory consumption is limited to buf_size.

`RingFile`: to record log content in memory for multi-threaded program. Act as an observer to
analyze concurrency problem. It maintain thread local buffer to avoid lock contention.
Already integrated into [captain-log](https://docs.rs/captains-log) as `LogRingFile` sink.
