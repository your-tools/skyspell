use skyspell_core::{
    Checker, ProjectFile,
    tests::{FakeDictionary, TestContext, get_test_context, get_test_dir},
};
use tempfile::TempDir;

use crate::{JsonChecker, checkers::json::Range};

type JsonTestChecker = JsonChecker<FakeDictionary>;

struct TestApp {
    checker: JsonTestChecker,
}

impl TestApp {
    fn new(temp_dir: &TempDir) -> Self {
        let context = get_test_context(temp_dir);
        let TestContext {
            project,
            ignore_store,
            dictionary,
            ..
        } = context;
        let checker = JsonTestChecker::new(project, dictionary, ignore_store).unwrap();
        Self { checker }
    }

    fn new_project_file(&self, path: &str) -> ProjectFile {
        let project = &self.checker.project;
        let project_path = project.path();
        let full_path = project_path.join(path);
        project.new_project_file(full_path).unwrap()
    }
}

#[test]
fn test_read_skipped_tokens() {
    let temp_dir = get_test_dir();
    let contents = "First line
SKIP_THIS
last line";
    let mut app = TestApp::new(&temp_dir);
    app.checker.dictionary.add_known("First");
    app.checker.dictionary.add_known("last");
    app.checker.dictionary.add_known("line");

    let foo_py_path = temp_dir.path().join("project/foo.py");
    std::fs::write(&foo_py_path, contents).unwrap();

    let foo_py = app.new_project_file("foo.py");
    app.checker
        .ignore_store()
        .skip_token("SKIP_THIS", &foo_py)
        .unwrap();
    app.checker.process(&foo_py_path, &()).unwrap();
    assert!(app.checker.spell_result.errors.is_empty());
}

#[test]
fn test_output() {
    let temp_dir = get_test_dir();

    let one = "first second";
    let two = "third fourth";
    let mut app = TestApp::new(&temp_dir);

    let one_txt = app.new_project_file("one.txt");
    std::fs::write(one_txt.full_path(), one).unwrap();

    let two_txt = app.new_project_file("two.txt");
    std::fs::write(two_txt.full_path(), two).unwrap();

    app.checker.dictionary.add_known("first");
    app.checker.dictionary.add_known("fourth");
    app.checker
        .dictionary
        .add_suggestions("second", &["s1".to_owned(), "s2".to_owned()]);

    app.checker.process(one_txt.full_path(), &()).unwrap();
    app.checker.process(two_txt.full_path(), &()).unwrap();

    app.checker.populate_result();
    let result = app.checker.spell_result;
    let expected_key = one_txt.full_path().to_string_lossy().into_owned();

    #[cfg(target_family = "windows")]
    let expected_key = expected_key.replace("/", "\\");

    let one_errors = &result.errors[&expected_key];
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
