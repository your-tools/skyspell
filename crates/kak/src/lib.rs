#![deny(clippy::unwrap_used)]
#[cfg(target_family = "unix")]
#[path = "unix.rs"]
mod kak;

#[cfg(target_family = "windows")]
#[path = "windows.rs"]
mod kak;

pub use kak::main;
