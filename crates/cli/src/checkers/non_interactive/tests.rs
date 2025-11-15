use skyspell_core::{Checker, IgnoreStore, Project, RelativePath, tests::FakeDictionary};
use tempfile::TempDir;

use crate::{NonInteractiveChecker, OutputFormat};

type TestChecker = NonInteractiveChecker<FakeDictionary>;
struct TestApp {
    checker: TestChecker,
}

impl TestApp {
    fn new(temp_dir: &TempDir) -> Self {
        let dictionary = FakeDictionary::new();

        let project_path = temp_dir.path().join("project");
        std::fs::create_dir(&project_path).unwrap();
        let project = Project::new(&project_path).unwrap();
        let global_toml = temp_dir.path().join("global.toml");
        let local_toml = temp_dir.path().join("skyspell.toml");
        let ignore_store = IgnoreStore::load(global_toml, local_toml).unwrap();
        let checker =
            TestChecker::new(project, dictionary, ignore_store, OutputFormat::Text).unwrap();
        Self { checker }
    }

    fn to_relative_path(&self, path: &str) -> RelativePath {
        let project_path = self.checker.project.path();
        let path = project_path.as_ref().join(path);
        RelativePath::new(project_path, &path).unwrap()
    }
}

#[test]
fn test_read_skipped_tokens() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let contents = "First line
SKIP_THIS
last line";
    let mut app = TestApp::new(&temp_dir);
    app.checker.dictionary.add_known("First");
    app.checker.dictionary.add_known("last");
    app.checker.dictionary.add_known("line");

    let foo_py_path = temp_dir.path().join("project/foo.py");
    std::fs::write(&foo_py_path, contents).unwrap();

    let foo_py = app.to_relative_path("foo.py");
    app.checker
        .ignore_store()
        .skip_token("SKIP_THIS", &foo_py)
        .unwrap();
    app.checker.process(&foo_py_path, &()).unwrap();
    assert!(app.checker.errors.is_empty());
}
