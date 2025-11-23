use std::cell::RefCell;
use std::collections::VecDeque;

use super::InteractiveChecker;
use crate::Interactor;
use skyspell_core::tests::{FakeDictionary, TestContext, get_test_context, get_test_dir};
use skyspell_core::{Checker, Position, ProjectFile};
use tempfile::TempDir;

#[derive(Debug)]
enum Answer {
    Text(String),
    Int(Option<usize>),
    Bool(bool),
}

#[derive(Debug, Default)]
pub struct FakeInteractor {
    answers: RefCell<VecDeque<Answer>>,
}

impl FakeInteractor {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn push_text(&self, text: &str) {
        self.answers
            .borrow_mut()
            .push_front(Answer::Text(text.to_string()))
    }

    pub fn push_int(&self, i: usize) {
        self.answers.borrow_mut().push_front(Answer::Int(Some(i)))
    }

    pub fn push_bool(&self, b: bool) {
        self.answers.borrow_mut().push_front(Answer::Bool(b))
    }

    pub fn is_empty(&self) -> bool {
        self.answers.borrow().is_empty()
    }
}

impl Interactor for FakeInteractor {
    fn input(&self, prompt: &str) -> String {
        println!("{prompt} >");
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recorded answer");
        match answer {
            Answer::Text(t) => {
                print!("> {t}");
                t
            }
            a => panic!("Should have got a text answer, got {a:?}"),
        }
    }

    fn input_letter(&self, prompt: &str, choices: &str) -> String {
        println!("{prompt}");
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recorded answer");
        match answer {
            Answer::Text(s) => {
                println!("> {s}");
                if !choices.contains(&s) {
                    panic!("should have got an answer matching the possible choices");
                }
                s
            }
            a => panic!("Should have got a text answer, got {a:?}"),
        }
    }

    fn select(&self, prompt: &str, choices: &[&str]) -> Option<usize> {
        for choice in choices {
            println!("{choice}");
        }
        println!("{prompt} >");
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recorded answer");
        match answer {
            Answer::Int(i) => {
                println!("> {i:?}");
                i
            }
            a => panic!("Should have got a int answer, got {a:?}"),
        }
    }

    fn confirm(&self, prompt: &str) -> bool {
        println!("{prompt} >");
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recorded answer");
        match answer {
            Answer::Bool(b) => {
                println!("> {b}");
                b
            }
            a => panic!("Should have got a boolean answer, got {a:?}"),
        }
    }
}

#[test]
fn test_fake_interactor_replay_recorded_answers() {
    let fake_interactor = FakeInteractor::new();
    fake_interactor.push_text("Alice");
    fake_interactor.push_text("blue");
    fake_interactor.push_int(1);
    fake_interactor.push_bool(true);
    fake_interactor.push_text("q");

    let name = fake_interactor.input("What is your name");
    let color = fake_interactor.input("What is your favorite color");
    let index = fake_interactor.select("Coffee or tea?", &["coffee", "tea"]);
    let sugar = fake_interactor.confirm("With sugar?");
    let quit = fake_interactor.input_letter("What now?", "qyn");

    assert_eq!(name, "Alice");
    assert_eq!(color, "blue");
    assert_eq!(index, Some(1));
    assert!(sugar);
    assert_eq!(quit, "q");
}

#[test]
#[should_panic]
fn test_fake_interactor_on_missing_answer() {
    let fake_interactor = FakeInteractor::new();
    fake_interactor.push_text("Alice");

    fake_interactor.input("What is your name");
    fake_interactor.input("What is your favorite color");
}

type TestChecker = InteractiveChecker<FakeInteractor, FakeDictionary>;

struct TestApp {
    checker: TestChecker,
}

impl TestApp {
    fn new(temp_dir: &TempDir) -> Self {
        let context = get_test_context(temp_dir);
        let TestContext {
            project,
            ignore_store,
            state_toml,
            dictionary,
            ..
        } = context;

        let interactor = FakeInteractor::new();
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
    let temp_dir = get_test_dir();
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
    let temp_dir = get_test_dir();
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
    let temp_dir = get_test_dir();
    let mut app = TestApp::new(&temp_dir);
    app.add_known(&["hello", "world"]);
    app.push_text("l");

    app.handle_token("foo", "hello.py");

    assert!(app.is_ignored_for_lang("foo", "en"));

    app.end();
}

#[test]
fn test_adding_word_to_project() {
    let temp_dir = get_test_dir();
    let mut app = TestApp::new(&temp_dir);
    app.push_text("p");

    app.handle_token("foo", "foo.py");

    assert!(app.is_ignored_for_project("foo"));
    app.handle_token("foo", "foo.py");

    app.end()
}

#[test]
fn test_ignore_word_to_project_file() {
    let temp_dir = get_test_dir();
    let mut app = TestApp::new(&temp_dir);
    app.push_text("f");

    app.handle_token("foo", "foo.py");

    assert!(app.is_ignored_for_path("foo", "foo.py"));
    app.handle_token("foo", "foo.py");

    app.end()
}

#[test]
fn test_remember_skipped_words() {
    let temp_dir = get_test_dir();
    let mut app = TestApp::new(&temp_dir);
    app.add_known(&["hello", "world"]);
    app.push_text("x");

    app.handle_token("foo", "foo.py");
    app.handle_token("foo", "foo.py");

    app.end();
}
