use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;

use crate::Dictionary;
use crate::Interactor;
use crate::Repo;

#[derive(Debug)]
enum Answer {
    Text(String),
    Int(Option<usize>),
    Bool(bool),
}

pub(crate) struct FakeInteractor {
    answers: RefCell<VecDeque<Answer>>,
}

impl Default for FakeInteractor {
    fn default() -> Self {
        let queue = VecDeque::new();
        Self {
            answers: RefCell::new(queue),
        }
    }
}

impl FakeInteractor {
    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub(crate) fn push_text(&self, text: &str) {
        self.answers
            .borrow_mut()
            .push_front(Answer::Text(text.to_string()))
    }

    pub(crate) fn push_int(&self, i: usize) {
        self.answers.borrow_mut().push_front(Answer::Int(Some(i)))
    }

    pub(crate) fn push_bool(&self, b: bool) {
        self.answers.borrow_mut().push_front(Answer::Bool(b))
    }
}

impl Drop for FakeInteractor {
    fn drop(&mut self) {
        if !self.answers.borrow().is_empty() {
            panic!("not all answers have been consumed by the tests");
        }
    }
}

impl Interactor for FakeInteractor {
    fn input(&self, prompt: &str) -> String {
        println!("{} >", prompt);
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recorded answer");
        match answer {
            Answer::Text(t) => {
                print!("> {}", t);
                t
            }
            a => panic!("Should have got a text answer, got {:?}", a),
        }
    }

    fn input_letter(&self, prompt: &str, choices: &str) -> String {
        println!("{}", prompt);
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recorded answer");
        match answer {
            Answer::Text(s) => {
                println!("> {}", s);
                if !choices.contains(&s) {
                    panic!("should have got an answer matching the possible choices");
                }
                s
            }
            a => panic!("Should have got a text answer, got {:?}", a),
        }
    }

    fn select(&self, prompt: &str, choices: &[&str]) -> Option<usize> {
        for choice in choices {
            println!("{}", choice);
        }
        println!("{} >", prompt);
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recorded answer");
        match answer {
            Answer::Int(i) => {
                println!("> {:?}", i);
                i
            }
            a => panic!("Should have got a int answer, got {:?}", a),
        }
    }

    fn confirm(&self, prompt: &str) -> bool {
        println!("{} >", prompt);
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recorded answer");
        match answer {
            Answer::Bool(b) => {
                println!("> {}", b);
                b
            }
            a => panic!("Should have got a boolean answer, got {:?}", a),
        }
    }
}

#[derive(Default)]
pub(crate) struct FakeRepo {
    good: HashSet<String>,
    ignored: HashSet<String>,
    skipped_file_names: HashSet<String>,
    skipped_paths: HashSet<String>,
    ignored_for_file: HashMap<String, Vec<String>>,
    ignored_for_ext: HashMap<String, Vec<String>>,
}

impl FakeRepo {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Repo for FakeRepo {
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()> {
        for word in words {
            self.ignored.insert(word.to_string());
        }
        Ok(())
    }

    fn add_ignored(&mut self, word: &str) -> Result<i32> {
        self.ignored.insert(word.to_string());
        Ok(0)
    }

    fn add_extension(&mut self, ext: &str) -> Result<()> {
        self.ignored_for_ext.insert(ext.to_string(), vec![]);
        Ok(())
    }

    fn add_file(&mut self, path: &str) -> Result<()> {
        self.ignored_for_file.insert(path.to_string(), vec![]);
        Ok(())
    }

    fn add_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()> {
        let entry = &mut self
            .ignored_for_ext
            .get_mut(ext)
            .ok_or_else(|| anyhow!("Unknown ext: {}", ext))?;
        entry.push(word.to_string());
        Ok(())
    }

    fn add_ignored_for_file(&mut self, word: &str, file: &str) -> Result<()> {
        let entry = self
            .ignored_for_file
            .get_mut(file)
            .ok_or_else(|| anyhow!("Unknown file: {}", file))?;
        entry.push(word.to_string());
        Ok(())
    }

    fn lookup_word(&self, word: &str, path: &Path) -> Result<bool> {
        let full_path = path.to_str();
        let ext = path.extension().and_then(|x| x.to_str());
        let file_name = path.file_name().and_then(|f| f.to_str());

        if self.good.contains(word) {
            return Ok(true);
        }

        if self.ignored.contains(word) {
            return Ok(true);
        }

        if let Some(f) = file_name {
            if self.skipped_file_names.contains(f) {
                return Ok(true);
            }
        }

        if let Some(ext) = ext {
            if let Some(for_ext) = self.ignored_for_ext.get(ext) {
                if for_ext.contains(&word.to_string()) {
                    return Ok(true);
                }
            }
        }

        if let Some(full_path) = full_path {
            if let Some(for_file) = self.ignored_for_file.get(full_path) {
                if for_file.contains(&word.to_string()) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn is_skipped(&self, path: &Path) -> Result<bool> {
        let full_path = path.to_str().unwrap();
        if self.skipped_paths.contains(full_path) {
            return Ok(true);
        }

        let file_name = path.file_name().unwrap().to_str().unwrap();
        if self.skipped_file_names.contains(file_name) {
            return Ok(true);
        }

        Ok(false)
    }

    fn known_extension(&self, ext: &str) -> Result<bool> {
        Ok(self.ignored_for_ext.contains_key(ext))
    }

    fn known_file(&self, full_path: &str) -> Result<bool> {
        Ok(self.ignored_for_file.contains_key(full_path))
    }

    fn skip_file_name(&mut self, filename: &str) -> Result<()> {
        self.skipped_file_names.insert(filename.to_string());
        Ok(())
    }

    fn skip_full_path(&mut self, full_path: &str) -> Result<()> {
        self.skipped_paths.insert(full_path.to_string());
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct FakeDictionary {
    known: Vec<String>,
    suggestions: HashMap<String, Vec<String>>,
}

impl FakeDictionary {
    fn new() -> Self {
        Default::default()
    }

    pub(crate) fn add_known(&mut self, word: &str) {
        self.known.push(word.to_string());
    }

    pub(crate) fn add_suggestions(&mut self, error: &str, suggestions: &[String]) {
        self.suggestions
            .insert(error.to_string(), suggestions.to_vec());
    }
}

impl Dictionary for FakeDictionary {
    fn check(&self, word: &str) -> Result<bool> {
        Ok(self.known.contains(&word.to_string()))
    }

    fn suggest(&self, error: &str) -> Vec<String> {
        self.suggestions.get(error).map_or(vec![], |v| v.to_vec())
    }
}

#[test]
fn test_fake_repo_lookup_ignored() {
    let mut fake = FakeRepo::new();
    fake.add_ignored("foobar").unwrap();

    assert!(fake.lookup_word("foobar", &Path::new("-")).unwrap())
}

#[test]
fn test_fake_repo_lookup_for_extension() {
    let mut fake = FakeRepo::new();
    fake.add_extension("py").unwrap();
    fake.add_ignored_for_extension("defaultdict", "py").unwrap();

    assert!(!fake
        .lookup_word("defaultdict", &Path::new("hello.rs"))
        .unwrap());
    assert!(fake
        .lookup_word("defaultdict", &Path::new("hello.py"))
        .unwrap());
}

#[test]
fn test_fake_repo_lookup_for_file() {
    let mut fake = FakeRepo::new();
    fake.add_file("path/to/foo.txt").unwrap();
    fake.add_ignored_for_file("abcdef", "path/to/foo.txt")
        .unwrap();

    assert!(fake
        .lookup_word("abcdef", &Path::new("path/to/foo.txt"))
        .unwrap());
    assert!(!fake
        .lookup_word("abcdef", &Path::new("path/to/other.txt"))
        .unwrap());
}

#[test]
fn test_fake_repo_skipping_filename() {
    let mut fake = FakeRepo::new();
    fake.skip_file_name("poetry.lock").unwrap();

    assert!(fake
        .lookup_word("abcdef", &Path::new("path/to/poetry.lock"))
        .unwrap());
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

#[test]
fn test_fake_dictionary_check() {
    let mut fake_dictionary = FakeDictionary::new();
    fake_dictionary.add_known("hello");

    assert!(fake_dictionary.check("hello").unwrap());
    assert!(!fake_dictionary.check("foo").unwrap());
}

#[test]
fn test_fake_dictionary_suggest() {
    let mut fake_dictionary = FakeDictionary::new();
    fake_dictionary.add_known("hello");
    fake_dictionary.add_suggestions("missstake", &["mistake".to_string()]);

    assert_eq!(&fake_dictionary.suggest("missstake"), &["mistake"]);
    assert!(&fake_dictionary.suggest("asntoehsauh").is_empty());
}
