#![allow(clippy::unwrap_used)] // this is test code, it's ok to unwrap
use tempfile::TempDir;

use crate::{ProjectPath, RelativePath};

pub mod fake_dictionary;
pub mod fake_io;
pub mod test_dictionary;
pub mod test_ignore_store;
pub mod test_repository;

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
