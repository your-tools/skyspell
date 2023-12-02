use super::*;

use skyspell_core::tests::FakeDictionary;
use skyspell_core::RelativePath;

use tempfile::TempDir;

mod fake_interactor;
pub use fake_interactor::FakeInteractor;

struct TestApp {
    dictionary: FakeDictionary,
    ignore_config: IgnoreConfig,
    project: Project,
}

impl TestApp {
    fn new(_temp_dir: &TempDir) -> Self {
        todo!()
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
        dbg!(&with_arg0);
        let opts = Opts::try_parse_from(with_arg0)?;
        super::run(self.project, &opts, self.dictionary, self.ignore_config)
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

    todo!()
}

#[test]
fn test_add_for_project_happy() {
    let _temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    todo!()
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

    todo!()
}

#[test]
fn test_add_for_relative_path() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);
    let (full_path, _rel_path) = app.ensure_file("foo.txt");

    app.run(&[
        "add",
        "foo",
        "--relative-path",
        &full_path.to_string_lossy(),
    ])
    .unwrap();

    todo!()
}

#[test]
fn test_remove_global() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.ignore_config.ignore("foo").unwrap();

    app.run(&["remove", "foo"]).unwrap();

    todo!()
}

#[test]
fn test_remove_for_project() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);

    app.run(&["remove", "foo", "--project"]).unwrap();

    todo!()
}

#[test]
fn test_remove_for_relative_path() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);
    let (_full_path, _rel_path) = app.ensure_file("foo.txt");

    todo!();
}

#[test]
fn test_remove_for_extension() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.ensure_file("foo.py");
    app.ignore_config.ignore_for_extension("foo", "py").unwrap();

    app.run(&["remove", "foo", "--extension", "py"]).unwrap();

    todo!()
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
fn test_reading_ignore_patterns_from_config() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let app = TestApp::new(&temp_dir);
    let (foo_full, _) = app.ensure_file("foo.lock");
    let (config_path, _) = app.ensure_file("skyspell-ignore.kdl");
    std::fs::write(foo_full, "error").unwrap();
    std::fs::write(config_path, "patterns {\n *.lock \n}\n").unwrap();

    app.run(&["check", "--non-interactive"]).unwrap();
}
