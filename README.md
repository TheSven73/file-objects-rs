# file-objects-rs

### Real and fake implementations of Rust `std::fs::File` objects.

[![Build Status](https://api.travis-ci.org/TheSven73/file-objects-rs.svg?branch=master)](https://travis-ci.com/github/TheSven73/file-objects-rs)

file-objects-rs provides real and fake implementations of file system-related
functionality. It abstracts away details of certain common but complex operations
(e.g., setting permissions) and makes it easier to test any file system-related
logic without having to wait for slow I/O operations or coerce the file system
into particular states.

file-objects-rs is a fork of Isobel Redelmeier's excellent
[filesystem-rs](https://crates.io/crates/filesystem) crate. Its custom API has
been replaced by objects closely mirroring Rust's `std::fs::File` objects.
