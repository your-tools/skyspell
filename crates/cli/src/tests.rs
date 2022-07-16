use super::*;

use skyspell_core::tests::{new_project_path, FakeDictionary};
use skyspell_core::IgnoreStore;
use skyspell_core::{ProjectPath, RelativePath};
use skyspell_core::{Repository, SQLRepository, StorageBackend};

use tempfile::TempDir;

mod fake_interactor;
pub use fake_interactor::FakeInteractor;

fn open_repository(temp_dir: &TempDir) -> SQLRepository {
    SQLRepository::new(&TestApp::db_path(temp_dir)).unwrap()
}

struct TestApp {
    dictionary: FakeDictionary,
    storage_backend: StorageBackend,
}

impl TestApp {
    fn new(temp_dir: &TempDir) -> Self {
        let dictionary = FakeDictionary::new();
        let db_path = Self::db_path(temp_dir);
        let repository = SQLRepository::new(&db_path).unwrap();
        let storage_backend = StorageBackend::Repository(Box::new(repository));
        Self {
            dictionary,
            storage_backend,
        }
    }

    fn new_project_path(&mut self, temp_dir: &TempDir, project_name: &str) -> ProjectPath {
        new_project_path(temp_dir, project_name)
    }

    fn ensure_file(
        temp_dir: &TempDir,
        project_name: &str,
        file_name: &str,
    ) -> (PathBuf, RelativePath) {
        let project = new_project_path(temp_dir, project_name);
        let full_path = project.as_ref().join(file_name);
        std::fs::write(&full_path, "").unwrap();
        (
            full_path.clone(),
            RelativePath::new(&project, &full_path).unwrap(),
        )
    }

    fn db_path(temp_dir: &TempDir) -> String {
        temp_dir
            .path()
            .join("tests.db")
            .to_string_lossy()
            .to_string()
    }

    fn run(self, args: &[&str]) -> Result<()> {
        let mut with_arg0 = vec!["skyspell"];
        with_arg0.extend(args);
        let opts = Opts::try_parse_from(with_arg0)?;
        super::run(opts, self.dictionary, self.storage_backend)
    }
}

#[test]
fn test_add_global() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);

    app.run(&["add", "foo"]).unwrap();

    let repository = open_repository(&temp_dir);
    assert!(repository.is_ignored("foo").unwrap());
}

#[test]
fn test_add_for_project_happy() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let project = app.new_project_path(&temp_dir, "project");

    app.run(&["add", "foo", "--project-path", &project.as_str()])
        .unwrap();

    let repository = open_repository(&temp_dir);
    let project_id = repository.get_project_id(&project).unwrap();
    assert!(repository
        .is_ignored_for_project("foo", project_id)
        .unwrap());
}

#[test]
fn test_add_for_extension() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);
    TestApp::ensure_file(&temp_dir, "project", "foo.py");

    app.run(&["add", "foo", "--extension", "py"]).unwrap();

    let repository = open_repository(&temp_dir);
    assert!(repository.is_ignored_for_extension("foo", "py").unwrap());
}

#[test]
fn test_add_for_relative_path() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let (full_path, rel_path) = TestApp::ensure_file(&temp_dir, "project", "foo.txt");
    let project = app.new_project_path(&temp_dir, "project");

    app.run(&[
        "add",
        "foo",
        "--project-path",
        &project.as_str(),
        "--relative-path",
        &full_path.to_string_lossy(),
    ])
    .unwrap();

    let repository = open_repository(&temp_dir);
    let project_id = repository.get_project_id(&project).unwrap();
    assert!(repository
        .is_ignored_for_path("foo", project_id, &rel_path)
        .unwrap());
}

#[test]
fn test_remove_global() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.storage_backend
        .ignore_store_mut()
        .ignore("foo")
        .unwrap();

    app.run(&["remove", "foo"]).unwrap();

    let repository = open_repository(&temp_dir);
    assert!(!repository.is_ignored("foo").unwrap());
}

#[test]
fn test_suggest() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.dictionary
        .add_suggestions("hel", &["hello".to_string(), "hell".to_string()]);

    app.run(&["suggest", "hel"]).unwrap();
}
