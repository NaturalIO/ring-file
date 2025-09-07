# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.1] 2025-09-08

### Fixed

- Fix RingFile::clear() signature

## [0.3.0] 2025-09-08

### Changed

- Refactor RingFile, with a background thread to maintain a RingBuffer.
Because in previous impl different thread might produce different amount of data,
Leaving some holes in the final dump.

## [0.2.2] 2025-09-07

### Added

- Added RingFile::clear()

## [0.2.0] 2025-09-06

### Changed

- Rename RingFile into RingBuffer

- Refactor with thread_local crate.
 The new RingFile will maintain thread local buffer and merge them by the order
 of timestamp on dump().

## [0.1.1] 2025-08-01

### Added

- First version


