use super::*;

use skyspell_core::tests::FakeDictionary;
use skyspell_core::{RelativePath, SKYSPELL_CONFIG_FILE};

use tempfile::TempDir;

mod fake_interactor;
pub use fake_interactor::FakeInteractor;

struct TestApp {
    dictionary: FakeDictionary,
    ignore_store: IgnoreStore,
    project: Project,
}

impl TestApp {
    fn new(temp_dir: &TempDir) -> Self {
        let dictionary = FakeDictionary::new();
        let project_path = temp_dir.path().join("project");
        std::fs::create_dir(&project_path).unwrap();
        let local_path = project_path.join(SKYSPELL_CONFIG_FILE);
        let preset_path = temp_dir.path().join("preset.toml");
        let ignore_store = IgnoreStore::load(preset_path, local_path).unwrap();
        let project_path = ProjectPath::new(&project_path).unwrap();
        let project = Project::new(project_path);
        Self {
            dictionary,
            ignore_store,
            project,
        }
    }

    fn load_store(temp_dir: &TempDir) -> IgnoreStore {
        let preset_path = temp_dir.path().join("preset.toml");
        let local_path = temp_dir.path().join("project").join(SKYSPELL_CONFIG_FILE);
        IgnoreStore::load(preset_path, local_path).unwrap()
    }

    fn ensure_file(&self, file_name: &str) -> (PathBuf, RelativePath) {
        let full_path = self.project.path().as_ref().join(file_name);
        std::fs::write(&full_path, "").unwrap();
        let relative_path = self.project.get_relative_path(&full_path).unwrap();
        (full_path, relative_path)
    }

    fn run(self, args: &[&str]) -> Result<()> {
        let project_path_as_str = self.project.as_str();
        let mut with_arg0 = vec!["skyspell"];
        with_arg0.push("--project-path");
        with_arg0.push(&project_path_as_str);
        with_arg0.extend(args);
        let opts = Opts::try_parse_from(with_arg0)?;
        super::run(self.project, &opts, self.dictionary, self.ignore_store)
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

    let store = TestApp::load_store(&temp_dir);
    assert!(store.is_ignored("foo"));
}

#[test]
fn test_add_for_project() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);

    app.run(&["add", "foo", "--project"]).unwrap();

    let store = TestApp::load_store(&temp_dir);
    assert!(store.is_ignored_for_project("foo"));
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

    let ignore_store = TestApp::load_store(&temp_dir);
    assert!(ignore_store.is_ignored_for_extension("foo", "py"));
}

#[test]
fn test_add_for_relative_path() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);
    let (full_path, rel_path) = app.ensure_file("foo.txt");

    app.run(&[
        "add",
        "foo",
        "--relative-path",
        &full_path.to_string_lossy(),
    ])
    .unwrap();

    let store = TestApp::load_store(&temp_dir);
    assert!(store.is_ignored_for_path("foo", &rel_path));
}

#[test]
fn test_remove_global() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.ignore_store.ignore("foo").unwrap();

    app.run(&["remove", "foo"]).unwrap();

    let store = TestApp::load_store(&temp_dir);

    assert!(!store.is_ignored("foo"));
}

#[test]
fn test_remove_for_project() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.ignore_store.ignore_for_project("foo").unwrap();

    app.run(&["remove", "foo", "--project"]).unwrap();

    let store = TestApp::load_store(&temp_dir);
    assert!(!store.is_ignored_for_project("foo"));
}

#[test]
fn test_remove_for_relative_path() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    let (full_path, rel_path) = app.ensure_file("foo.txt");
    app.ignore_store.ignore_for_path("foo", &rel_path).unwrap();

    app.run(&[
        "remove",
        "foo",
        "--relative-path",
        &full_path.to_string_lossy(),
    ])
    .unwrap();

    let store = TestApp::load_store(&temp_dir);
    assert!(!store.is_ignored_for_path("foo", &rel_path));
}

#[test]
fn test_remove_for_extension() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.ensure_file("foo.py");
    app.ignore_store.ignore_for_extension("foo", "py").unwrap();

    app.run(&["remove", "foo", "--extension", "py"]).unwrap();

    let store = TestApp::load_store(&temp_dir);
    assert!(!store.is_ignored_for_extension("foo", "py"));
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
    std::fs::write(foo_full, "This is foo").unwrap();
    std::fs::write(bar_full, "This is bar and it contains baz").unwrap();
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
    std::fs::write(foo_full, "This is fine").unwrap();
    std::fs::write(bar_full, "This is also fine").unwrap();
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
fn test_reading_ignore_patterns_from_store() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);
    let (foo_full, _) = app.ensure_file("foo.lock");
    let (local_path, _) = app.ensure_file(SKYSPELL_CONFIG_FILE);
    std::fs::write(foo_full, "error").unwrap();
    std::fs::write(
        local_path,
        r#"
        patterns = ["*.lock"]
        "#,
    )
    .unwrap();

    app.run(&["check", "--non-interactive"]).unwrap();
}
