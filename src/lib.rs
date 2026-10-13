//! glassmon: a fast, good-looking terminal system monitor for macOS and Linux.
//!
//! The crate is split into a library (so integration tests and snapshot tests
//! can drive the UI) and a thin binary in `main.rs`.

#![forbid(unsafe_code)]

pub mod tui;
