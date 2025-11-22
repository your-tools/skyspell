#![allow(clippy::unwrap_used)] // this is test code, it's ok to unwrap
#![allow(dead_code)] // we have a public core::tests module that is only used by tests
use tempfile::TempDir;

use crate::IgnoreStore;

pub mod fake_dictionary;
pub mod fake_io;

pub use fake_dictionary::FakeDictionary;
pub use fake_io::FakeIO;

pub(crate) fn get_test_dir() -> TempDir {
    tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap()
}

pub(crate) fn create_store(temp_dir: &TempDir, global: &str, local: &str) -> IgnoreStore {
    let temp_path = temp_dir.path();
    let global_toml = temp_path.join("global.toml");
    std::fs::write(&global_toml, global).unwrap();
    let local_toml = temp_path.join("skyspell.toml");
    std::fs::write(&local_toml, local).unwrap();
    IgnoreStore::load(global_toml, local_toml).unwrap()
}

pub(crate) fn get_empty_store(temp_dir: &TempDir) -> IgnoreStore {
    create_store(temp_dir, "", "")
}
