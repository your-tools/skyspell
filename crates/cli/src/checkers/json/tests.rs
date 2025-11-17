use skyspell_core::{Checker, IgnoreStore, Project, RelativePath, tests::FakeDictionary};
use tempfile::TempDir;

use crate::{JsonChecker, checkers::json::Range};

type JsonTestChecker = JsonChecker<FakeDictionary>;

struct TestApp {
    checker: JsonTestChecker,
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
        let checker = JsonTestChecker::new(project, dictionary, ignore_store).unwrap();
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
    assert!(app.checker.spell_result.errors.is_empty());
}

#[test]
fn test_output() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();

    let one = "first second";
    let two = "third fourth";
    let mut app = TestApp::new(&temp_dir);

    let one_path = temp_dir.path().join("project/one.txt");
    std::fs::write(&one_path, one).unwrap();

    let two_path = temp_dir.path().join("project/two.txt");
    std::fs::write(&two_path, two).unwrap();

    app.checker.dictionary.add_known("first");
    app.checker.dictionary.add_known("fourth");
    app.checker
        .dictionary
        .add_suggestions("second", &["s1".to_owned(), "s2".to_owned()]);

    app.checker.process(&one_path, &()).unwrap();
    app.checker.process(&two_path, &()).unwrap();

    app.checker.populate_result();
    let result = app.checker.spell_result;
    let path: &str = &one_path.to_string_lossy();

    #[cfg(target_family = "windows")]
    let path = &path.replace("/", "\\");

    let one_errors = &result.errors[path];
    let first_error = &one_errors[0];
    assert_eq!(first_error.word, "second");
    assert_eq!(
        first_error.range,
        Range {
            line: 1,
            start_column: 7,
            end_column: 12
        }
    );

    let suggestions = &result.suggestions;
    assert_eq!(suggestions["second"], &["s1", "s2"]);
}
