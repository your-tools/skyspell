use tempfile::TempDir;

use super::InteractiveChecker;
use skyspell_core::tests::{FakeDictionary, FakeRepository};
use skyspell_core::{Checker, ProjectPath, RelativePath, StorageBackend};

use crate::tests::FakeInteractor;

type TestChecker = InteractiveChecker<FakeInteractor, FakeDictionary>;

struct TestApp {
    checker: TestChecker,
}

impl TestApp {
    fn new(temp_dir: &TempDir) -> Self {
        let interactor = FakeInteractor::new();
        let dictionary = FakeDictionary::new();
        let repository = FakeRepository::new();
        let project_path = ProjectPath::new(temp_dir.path()).unwrap();
        let mut storage_backend = StorageBackend::Repository(Box::new(repository));
        let project = storage_backend.ensure_project(&project_path).unwrap();
        let checker = TestChecker::new(project, interactor, dictionary, storage_backend).unwrap();
        Self { checker }
    }

    fn add_known(&mut self, words: &[&str]) {
        for word in words.iter() {
            self.checker.dictionary.add_known(word);
        }
    }

    fn push_text(&mut self, answer: &str) {
        self.checker.interactor.push_text(answer)
    }

    fn to_relative_path(&self, path: &str) -> RelativePath {
        let project_path = self.checker.project.path();
        let path = project_path.as_ref().join(path);
        RelativePath::new(project_path, &path).unwrap()
    }

    fn handle_token(&mut self, token: &str, relative_name: &str) {
        let project_path = self.checker.project().path();
        let full_path = project_path.as_ref().join(relative_name);
        std::fs::write(full_path, "").unwrap();
        let relative_path = self.to_relative_path(relative_name);
        let context = &(3, 42);
        self.checker
            .handle_token(token, &relative_path, context)
            .unwrap()
    }

    fn is_ignored(&mut self, word: &str) -> bool {
        self.checker.storage_backend().is_ignored(word).unwrap()
    }

    fn is_ignored_for_extension(&mut self, word: &str, extension: &str) -> bool {
        self.checker
            .storage_backend()
            .is_ignored_for_extension(word, extension)
            .unwrap()
    }

    fn is_ignored_for_project(&mut self, word: &str) -> bool {
        let project_id = self.checker.project().id();
        self.checker
            .storage_backend()
            .is_ignored_for_project(word, project_id)
            .unwrap()
    }

    fn is_ignored_for_path(&mut self, word: &str, relative_name: &str) -> bool {
        let project_id = self.checker.project().id();
        let relative_path = self.to_relative_path(relative_name);
        self.checker
            .storage_backend()
            .is_ignored_for_path(word, project_id, &relative_path)
            .unwrap()
    }

    fn end(&self) {
        if !self.checker.interactor.is_empty() {
            panic!("Not all answered consumed by the test");
        }
    }
}

#[test]
fn test_adding_token_to_global_ignore() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.add_known(&["hello", "world"]);
    app.push_text("a");

    app.handle_token("foo", "foo.txt");

    assert!(app.is_ignored("foo"));
    app.handle_token("foo", "other.ext");

    app.end();
}

#[test]
fn test_adding_token_to_extension() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.add_known(&["hello", "world"]);
    app.push_text("e");

    app.handle_token("defaultdict", "hello.py");

    assert!(app.is_ignored_for_extension("defaultdict", "py"));
    app.handle_token("defaultdict", "bar.py");

    app.end();
}

#[test]
fn test_adding_token_to_project() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.push_text("p");

    app.handle_token("foo", "foo.py");

    assert!(app.is_ignored_for_project("foo"));
    app.handle_token("foo", "foo.py");

    app.end()
}

#[test]
fn test_ignore_token_to_project_file() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.push_text("f");

    app.handle_token("foo", "foo.py");

    assert!(app.is_ignored_for_path("foo", "foo.py"));
    app.handle_token("foo", "foo.py");

    app.end()
}

#[test]
fn test_remember_skipped_tokens() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.add_known(&["hello", "world"]);
    app.push_text("x");

    app.handle_token("foo", "foo.py");
    app.handle_token("foo", "foo.py");

    app.end();
}
