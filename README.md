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

This crate provides a ringbuffer like file, in order to store byte content.
Already integrated into [captain-log](https://docs.rs/captains-log) as `LogRingFile` sink.

The content is kept in memory when written, when offset rewinds, new content will overwrite old content,
So that memory consumption is limited to buf_size.
Once deadlock encountered and process hangs, no more message will be written,
you can safely dump the content to disk.

# Example:

```rust
use ring_file::RingFile;
let mut file = RingFile::new(512*1024*1024, "/tmp/ringfile.log");
file.write_all("log message").expect("write ok");
file.dump().expect("dump ok");
```
