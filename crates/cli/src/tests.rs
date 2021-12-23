use super::*;

use skyspell_core::IgnoreStore;
use skyspell_core::{ProjectPath, RelativePath};
use skyspell_sql::SQLRepository;
use skyspell_tests::{new_project_path, FakeDictionary};

use tempfile::TempDir;

fn open_repository(temp_dir: &TempDir) -> SQLRepository {
    SQLRepository::new(&TestApp::db_path(temp_dir)).unwrap()
}

struct TestApp {
    dictionary: FakeDictionary,
    repository: SQLRepository,
}

impl TestApp {
    fn new(temp_dir: &TempDir) -> Self {
        let dictionary = FakeDictionary::new();
        let db_path = Self::db_path(temp_dir);
        let repository = SQLRepository::new(&db_path).unwrap();
        Self {
            dictionary,
            repository,
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
        let full_path = temp_dir.path().join(file_name);
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
        let opts = Opts::parse_from(with_arg0);
        super::run(opts, self.dictionary, self.repository)
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
    app.repository.ignore("foo").unwrap();

    app.run(&["remove", "foo"]).unwrap();

    let repository = open_repository(&temp_dir);
    assert!(!repository.is_ignored("foo").unwrap());
}

#[test]
fn test_remove_for_project() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let project = app.new_project_path(&temp_dir, "project");
    app.repository.new_project(&project).unwrap();
    let project_id = app.repository.get_project_id(&project).unwrap();
    app.repository
        .ignore_for_project("foo", project_id)
        .unwrap();

    app.run(&["remove", "foo", "--project-path", &project.as_str()])
        .unwrap();

    let repository = open_repository(&temp_dir);
    let project_id = repository.get_project_id(&project).unwrap();
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
    let (full_path, rel_path) = TestApp::ensure_file(&temp_dir, "project", "foo.txt");
    let project = app.new_project_path(&temp_dir, "project");
    let project_id = app.repository.new_project(&project).unwrap();
    app.repository
        .ignore_for_path("foo", project_id, &rel_path)
        .unwrap();

    app.run(&[
        "remove",
        "foo",
        "--project-path",
        &project.as_str(),
        "--relative-path",
        &full_path.to_string_lossy(),
    ])
    .unwrap();

    let repository = open_repository(&temp_dir);
    let project_id = repository.get_project_id(&project).unwrap();
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
    TestApp::ensure_file(&temp_dir, "project", "foo.py");
    app.repository.ignore_for_extension("foo", "py").unwrap();

    app.run(&["remove", "foo", "--extension", "py"]).unwrap();

    let repository = open_repository(&temp_dir);
    assert!(!repository.is_ignored_for_extension("foo", "py").unwrap());
}

#[test]
fn test_check_errors_in_two_files() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let project = app.new_project_path(&temp_dir, "project");
    let (foo_full, _) = TestApp::ensure_file(&temp_dir, "project", "foo.md");
    let (bar_full, _) = TestApp::ensure_file(&temp_dir, "project", "bar.md");
    std::fs::write(&foo_full, "This is foo").unwrap();
    std::fs::write(&bar_full, "This is bar and it contains baz").unwrap();
    for word in &["This", "is", "and", "it", "contains"] {
        app.dictionary.add_known(word);
    }

    let err = app
        .run(&[
            "check",
            "--non-interactive",
            "--project-path",
            &project.as_str(),
            &bar_full.to_string_lossy(),
            &foo_full.to_string_lossy(),
        ])
        .unwrap_err();

    assert!(err.to_string().contains("spelling errors"))
}

#[test]
fn test_check_happy() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let project = app.new_project_path(&temp_dir, "project");
    let (foo_full, _) = TestApp::ensure_file(&temp_dir, "project", "foo.md");
    let (bar_full, _) = TestApp::ensure_file(&temp_dir, "project", "bar.md");
    std::fs::write(&foo_full, "This is fine").unwrap();
    std::fs::write(&bar_full, "This is also fine").unwrap();
    for word in &["This", "is", "also", "fine"] {
        app.dictionary.add_known(word);
    }

    app.run(&[
        "check",
        "--non-interactive",
        "--project-path",
        &project.as_str(),
        &bar_full.to_string_lossy(),
        &foo_full.to_string_lossy(),
    ])
    .unwrap();
}

#[test]
fn test_skip_relative_path() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let (full_path, rel_path) = TestApp::ensure_file(&temp_dir, "project", "foo.txt");
    let project = app.new_project_path(&temp_dir, "project");

    app.run(&[
        "skip",
        "--project-path",
        &project.as_str(),
        "--relative-path",
        &full_path.to_string_lossy(),
    ])
    .unwrap();

    let repository = open_repository(&temp_dir);
    let project_id = repository.get_project_id(&project).unwrap();
    assert!(repository.is_skipped_path(project_id, &rel_path).unwrap());
}

#[test]
fn test_skip_file_name() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);

    app.run(&["skip", "--file-name", "Cargo.lock"]).unwrap();

    let repository = open_repository(&temp_dir);
    assert!(repository.is_skipped_file_name("Cargo.lock").unwrap());
}

#[test]
fn test_unskip_relative_path() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let (full_path, rel_path) = TestApp::ensure_file(&temp_dir, "project", "foo.txt");
    let project = app.new_project_path(&temp_dir, "project");
    let project_id = app.repository.new_project(&project).unwrap();
    app.repository.skip_path(project_id, &rel_path).unwrap();

    app.run(&[
        "unskip",
        "--project-path",
        &project.as_str(),
        "--relative-path",
        &full_path.to_string_lossy(),
    ])
    .unwrap();

    let repository = open_repository(&temp_dir);
    assert!(!repository.is_skipped_path(project_id, &rel_path).unwrap());
}

#[test]
fn test_unskip_file_name() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.repository.skip_file_name("Cargo.lock").unwrap();

    app.run(&["unskip", "--file-name", "Cargo.lock"]).unwrap();

    let repository = open_repository(&temp_dir);
    assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());
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
fn test_clean() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let project1 = app.new_project_path(&temp_dir, "project1");
    app.repository.new_project(&project1).unwrap();
    let project2 = app.new_project_path(&temp_dir, "project2");
    app.repository.new_project(&project2).unwrap();
    let before = app.repository.projects().unwrap();

    std::fs::remove_dir_all(&project2.as_ref()).unwrap();

    app.run(&["clean"]).unwrap();

    let repository = open_repository(&temp_dir);
    let after = repository.projects().unwrap();

    assert_eq!(
        before.len() - after.len(),
        1,
        "Should have removed one project"
    );
}