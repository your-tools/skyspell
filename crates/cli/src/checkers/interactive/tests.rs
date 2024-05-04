use super::InteractiveChecker;
use crate::tests::FakeInteractor;
use skyspell_core::tests::FakeDictionary;
use skyspell_core::{Checker, Config, Project, ProjectPath, RelativePath, SKYSPELL_CONFIG_FILE};
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
        let config_path = project_path.join(SKYSPELL_CONFIG_FILE);
        let project_path = ProjectPath::new(&project_path).unwrap();
        let project = Project::new(project_path);
        let ignore_config = Config::open_or_create(&config_path).unwrap();
        let state_toml = temp_dir.path().join("state.toml");
        let checker = TestChecker::new(
            project,
            interactor,
            dictionary,
            ignore_config,
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
        self.checker.ignore_config().is_ignored(word).unwrap()
    }

    fn is_ignored_for_extension(&mut self, word: &str, extension: &str) -> bool {
        self.checker
            .ignore_config()
            .is_ignored_for_extension(word, extension)
            .unwrap()
    }

    fn is_ignored_for_project(&mut self, word: &str) -> bool {
        self.checker
            .ignore_config()
            .is_ignored_for_project(word)
            .unwrap()
    }

    fn is_ignored_for_path(&mut self, word: &str, relative_name: &str) -> bool {
        let relative_path = self.to_relative_path(relative_name);
        self.checker
            .ignore_config()
            .is_ignored_for_path(word, &relative_path)
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
