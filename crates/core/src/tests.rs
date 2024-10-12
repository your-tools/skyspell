#![allow(clippy::unwrap_used)] // this is test code, it's ok to unwrap
#![allow(dead_code)] // we have a public core::tests module that is only used by tests
use tempfile::TempDir;

use crate::{IgnoreStore, ProjectPath, RelativePath};

pub mod fake_dictionary;
pub mod fake_io;

pub use fake_dictionary::FakeDictionary;
pub use fake_io::FakeIO;

pub fn new_project_path(temp_dir: &TempDir, name: &str) -> ProjectPath {
    let path = temp_dir.path().join(name);
    std::fs::create_dir_all(&path).unwrap();
    ProjectPath::new(&path).unwrap()
}

pub fn new_relative_path(project_path: &ProjectPath, name: &'static str) -> RelativePath {
    let rel_path = project_path.as_ref().join(name);
    std::fs::write(&rel_path, "").unwrap();
    RelativePath::new(project_path, &rel_path).unwrap()
}

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

pub(crate) fn relative_path(path: &str) -> RelativePath {
    RelativePath::from_path_unchecked(path.into())
}
