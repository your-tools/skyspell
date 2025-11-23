// Note: this is some helper code for tests of other crates.
#![allow(clippy::unwrap_used)]
#![allow(dead_code)]

use std::path::PathBuf;

use tempfile::TempDir;

use crate::{IgnoreStore, Project, SKYSPELL_LOCAL_IGNORE};

pub mod fake_dictionary;
pub mod fake_io;

pub use fake_dictionary::FakeDictionary;
pub use fake_io::FakeIO;

pub fn get_test_dir() -> TempDir {
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

#[non_exhaustive]
pub struct TestContext {
    pub project: Project,
    pub ignore_store: IgnoreStore,
    pub state_toml: PathBuf,
    pub dictionary: FakeDictionary,
}

pub fn get_test_context(temp_dir: &TempDir) -> TestContext {
    let project_path = temp_dir.path().join("project");
    std::fs::create_dir_all(&project_path).unwrap();
    let project = Project::new(&project_path).unwrap();
    let state_toml = temp_dir.path().join("state.toml");
    let global_toml = temp_dir.path().join("global.toml");
    let local_toml = temp_dir.path().join("project").join(SKYSPELL_LOCAL_IGNORE);
    let ignore_store = IgnoreStore::load(global_toml, local_toml).unwrap();
    let dictionary = FakeDictionary::new();

    TestContext {
        project,
        ignore_store,
        state_toml,
        dictionary,
    }
}
