use super::InteractiveChecker;
use crate::tests::FakeInteractor;
use skyspell_core::tests::FakeDictionary;
use skyspell_core::{Checker, IgnoreStore, Position, Project, ProjectFile};
use tempfile::TempDir;

type TestChecker = InteractiveChecker<FakeInteractor, FakeDictionary>;

struct TestApp {
    checker: TestChecker,
}

impl TestApp {
    fn new(temp_dir: &TempDir) -> Self {
        let dictionary = FakeDictionary::new();
        let interactor = FakeInteractor::new();

        let project_path = temp_dir.path().join("project");
        std::fs::create_dir(&project_path).unwrap();
        let project = Project::new(&project_path).unwrap();
        let state_toml = temp_dir.path().join("state.toml");
        let global_toml = temp_dir.path().join("global.toml");
        let local_toml = temp_dir.path().join("skyspell.toml");
        let ignore_store = IgnoreStore::load(global_toml, local_toml).unwrap();
        let checker = TestChecker::new(
            project,
            interactor,
            dictionary,
            ignore_store,
            Some(state_toml),
        )
        .unwrap();
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

    fn new_project_file(&self, path: &str) -> ProjectFile {
        let project_path = self.checker.project.path();
        let path = project_path.join(path);
        ProjectFile::new(self.checker.project(), &path).unwrap()
    }

    fn handle_token(&mut self, token: &str, relative_name: &str) {
        let project_path = self.checker.project().path();
        let full_path = project_path.join(relative_name);
        std::fs::write(full_path, "").unwrap();
        let project_file = self.new_project_file(relative_name);
        self.checker
            .handle_token(
                token,
                &project_file,
                Position {
                    line: 3,
                    column: 42,
                },
                &(),
            )
            .unwrap()
    }

    fn is_ignored(&mut self, word: &str) -> bool {
        self.checker.ignore_store().is_ignored(word)
    }

    fn is_ignored_for_extension(&mut self, word: &str, extension: &str) -> bool {
        self.checker
            .ignore_store()
            .is_ignored_for_extension(word, extension)
    }

    fn is_ignored_for_lang(&mut self, word: &str, lang: &str) -> bool {
        self.checker.ignore_store().is_ignored_for_lang(word, lang)
    }

    fn is_ignored_for_project(&mut self, word: &str) -> bool {
        self.checker.ignore_store().is_ignored_for_project(word)
    }

    fn is_ignored_for_path(&mut self, word: &str, name: &str) -> bool {
        let project_file = self.new_project_file(name);
        self.checker
            .ignore_store()
            .is_ignored_for_path(word, &project_file)
    }

    fn end(&self) {
        if !self.checker.interactor.is_empty() {
            panic!("Not all answered consumed by the test");
        }
    }
}

#[test]
fn test_adding_word_to_global_ignore() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.add_known(&["hello", "world"]);
    app.push_text("g");

    app.handle_token("foo", "foo.txt");

    assert!(app.is_ignored("foo"));
    app.handle_token("foo", "other.ext");

    app.end();
}

#[test]
fn test_adding_word_to_extension() {
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
fn test_adding_word_to_lang() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut app = TestApp::new(&temp_dir);
    app.add_known(&["hello", "world"]);
    app.push_text("l");

    app.handle_token("foo", "hello.py");

    assert!(app.is_ignored_for_lang("foo", "en"));

    app.end();
}

#[test]
fn test_adding_word_to_project() {
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
fn test_ignore_word_to_project_file() {
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
fn test_remember_skipped_words() {
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
