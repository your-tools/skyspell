use super::*;

use skyspell_core::tests::FakeDictionary;
use skyspell_core::IgnoreStore;
use skyspell_core::{ProjectId, ProjectPath, RelativePath};
use skyspell_core::{SQLRepository, StorageBackend};

use tempfile::TempDir;

mod fake_interactor;
pub use fake_interactor::FakeInteractor;

fn open_repository(temp_dir: &TempDir) -> SQLRepository {
    SQLRepository::new(&TestApp::db_path(temp_dir)).unwrap()
}

struct TestApp {
    dictionary: FakeDictionary,
    storage_backend: StorageBackend,
    project: Project,
}

impl TestApp {
    fn new(temp_dir: &TempDir) -> Self {
        let dictionary = FakeDictionary::new();
        let db_path = Self::db_path(temp_dir);
        let repository = SQLRepository::new(&db_path).unwrap();
        let mut storage_backend = StorageBackend::Repository(Box::new(repository));

        let project_path = temp_dir.path().join("project");
        std::fs::create_dir(&project_path).unwrap();
        let project_path = ProjectPath::new(&project_path).unwrap();
        let project = storage_backend.ensure_project(&project_path).unwrap();

        Self {
            dictionary,
            storage_backend,
            project,
        }
    }

    fn project_id(&self) -> ProjectId {
        self.project.id()
    }

    fn ensure_file(&self, file_name: &str) -> (PathBuf, RelativePath) {
        let full_path = self.project.path().as_ref().join(file_name);
        std::fs::write(&full_path, "").unwrap();
        let relative_path = self.project.get_relative_path(&full_path).unwrap();
        (full_path, relative_path)
    }

    fn db_path(temp_dir: &TempDir) -> String {
        temp_dir
            .path()
            .join("tests.db")
            .to_string_lossy()
            .to_string()
    }

    fn run(self, args: &[&str]) -> Result<()> {
        let project_path_as_str = self.project.as_str();
        let mut with_arg0 = vec!["skyspell"];
        with_arg0.push("--project-path");
        with_arg0.push(&project_path_as_str);
        with_arg0.extend(args);
        dbg!(&with_arg0);
        let opts = Opts::try_parse_from(with_arg0)?;
        super::run(self.project, &opts, self.dictionary, self.storage_backend)
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

    let mut repository = open_repository(&temp_dir);
    assert!(repository.is_ignored("foo").unwrap());
}

#[test]
fn test_add_for_project_happy() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);
    let project_id = app.project_id();
    app.run(&["add", "foo", "--project"]).unwrap();

    let mut repository = open_repository(&temp_dir);
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
    app.ensure_file("foo.py");

    app.run(&["add", "foo", "--extension", "py"]).unwrap();

    let mut repository = open_repository(&temp_dir);
    assert!(repository.is_ignored_for_extension("foo", "py").unwrap());
}

#[test]
fn test_add_for_relative_path() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);
    let project_id = app.project_id();
    let (full_path, rel_path) = app.ensure_file("foo.txt");

    app.run(&[
        "add",
        "foo",
        "--relative-path",
        &full_path.to_string_lossy(),
    ])
    .unwrap();

    let mut repository = open_repository(&temp_dir);
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

    let mut repository = open_repository(&temp_dir);
    assert!(!repository.is_ignored("foo").unwrap());
}

#[test]
fn test_remove_for_project() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let project_id = app.project_id();
    app.storage_backend
        .ignore_for_project("foo", project_id)
        .unwrap();

    app.run(&["remove", "foo", "--project"]).unwrap();

    let mut repository = open_repository(&temp_dir);
    assert!(!repository
        .is_ignored_for_project("foo", project_id)
        .unwrap());
}

#[test]
fn test_remove_for_relative_path() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let project_id = app.project_id();
    let (full_path, rel_path) = app.ensure_file("foo.txt");
    app.storage_backend
        .ignore_for_path("foo", project_id, &rel_path)
        .unwrap();

    app.run(&[
        "remove",
        "foo",
        "--relative-path",
        &full_path.to_string_lossy(),
    ])
    .unwrap();

    let mut repository = open_repository(&temp_dir);
    assert!(!repository
        .is_ignored_for_path("foo", project_id, &rel_path)
        .unwrap());
}

#[test]
fn test_remove_for_extension() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.ensure_file("foo.py");
    app.storage_backend
        .ignore_for_extension("foo", "py")
        .unwrap();

    app.run(&["remove", "foo", "--extension", "py"]).unwrap();

    let mut repository = open_repository(&temp_dir);
    assert!(!repository.is_ignored_for_extension("foo", "py").unwrap());
}

#[test]
fn test_check_errors_in_two_files() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let (foo_full, _) = app.ensure_file("foo.md");
    let (bar_full, _) = app.ensure_file("bar.md");
    std::fs::write(&foo_full, "This is foo").unwrap();
    std::fs::write(&bar_full, "This is bar and it contains baz").unwrap();
    for word in &["This", "is", "and", "it", "contains"] {
        app.dictionary.add_known(word);
    }

    let err = app.run(&["check", "--non-interactive"]).unwrap_err();

    assert!(err.to_string().contains("spelling errors"))
}

#[test]
fn test_check_happy() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let (foo_full, _) = app.ensure_file("foo.md");
    let (bar_full, _) = app.ensure_file("bar.md");
    std::fs::write(&foo_full, "This is fine").unwrap();
    std::fs::write(&bar_full, "This is also fine").unwrap();
    for word in &["This", "is", "also", "fine"] {
        app.dictionary.add_known(word);
    }

    app.run(&["check", "--non-interactive"]).unwrap();
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

#[test]
fn test_reading_ignore_patterns_from_config() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);
    let (foo_full, _) = app.ensure_file("foo.lock");
    let (config_path, _) = app.ensure_file("skyspell-ignore.kdl");
    std::fs::write(&foo_full, "error").unwrap();
    std::fs::write(&config_path, "patterns {\n *.lock \n}\n").unwrap();

    app.run(&["check", "--non-interactive"]).unwrap();
}
