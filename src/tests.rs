use anyhow::{anyhow, Result};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet, VecDeque};

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

impl FakeInteractor {
    pub(crate) fn new() -> Self {
        let queue = VecDeque::new();
        Self {
            answers: RefCell::new(queue),
        }
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

    pub(crate) fn assert_empty(&self) {
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
            .expect("should have got a recoreded answer");
        match answer {
            Answer::Text(t) => {
                print!("> {}", t);
                t
            }
            a => panic!("Should have got a text anwser, got {:?}", a),
        }
    }

    fn input_letter(&self, prompt: &str, choices: &str) -> String {
        println!("{}", prompt);
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recoreded answer");
        match answer {
            Answer::Text(s) => {
                println!("> {}", s);
                if !choices.contains(&s) {
                    panic!("should have got an answer maching the possible choices");
                }
                s
            }
            a => panic!("Should have got a text anwser, got {:?}", a),
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
            .expect("should have got a recoreded answer");
        match answer {
            Answer::Int(i) => {
                println!("> {:?}", i);
                i
            }
            a => panic!("Should have got a int anwser, got {:?}", a),
        }
    }

    fn confirm(&self, prompt: &str) -> bool {
        println!("{} >", prompt);
        let answer = self
            .answers
            .borrow_mut()
            .pop_back()
            .expect("should have got a recoreded answer");
        match answer {
            Answer::Bool(b) => {
                println!("> {}", b);
                b
            }
            a => panic!("Should have got a boolean anwser, got {:?}", a),
        }
    }
}

pub(crate) struct FakeRepo {
    good: HashSet<String>,
    ignored: HashSet<String>,
    ignored_for_file: HashMap<String, Vec<String>>,
    ignored_for_ext: HashMap<String, Vec<String>>,
}

impl FakeRepo {
    pub(crate) fn new() -> Self {
        Self {
            good: HashSet::new(),
            ignored: HashSet::new(),
            ignored_for_file: HashMap::new(),
            ignored_for_ext: HashMap::new(),
        }
    }
}

impl Repo for FakeRepo {
    fn add_good_words(&mut self, words: &[&str]) -> Result<()> {
        for word in words {
            self.good.insert(word.to_string());
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

    fn lookup_word(&self, word: &str, file: Option<&str>, ext: Option<&str>) -> Result<bool> {
        if self.good.contains(word) {
            return Ok(true);
        }

        if self.ignored.contains(word) {
            return Ok(true);
        }

        if let Some(ext) = ext {
            if let Some(for_ext) = self.ignored_for_ext.get(ext) {
                if for_ext.contains(&word.to_string()) {
                    return Ok(true);
                }
            }
        }

        if let Some(file) = file {
            if let Some(for_file) = self.ignored_for_file.get(file) {
                if for_file.contains(&word.to_string()) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn known_extension(&self, extension: &str) -> Result<bool> {
        Ok(self.ignored_for_ext.contains_key(extension))
    }

    fn known_file(&self, file: &str) -> Result<bool> {
        Ok(self.ignored_for_file.contains_key(file))
    }

    fn has_good_words(&self) -> Result<bool> {
        Ok(!self.good.is_empty())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_fake_repo_lookup_in_good_words() {
        let mut fake = FakeRepo::new();
        fake.add_good_words(&["hello", "hi"]).unwrap();

        assert!(fake.lookup_word("hello", None, None).unwrap());
        assert!(!fake.lookup_word("missstake", None, None).unwrap());
    }

    #[test]
    fn test_fake_repo_lookup_ignored() {
        let mut fake = FakeRepo::new();
        fake.add_good_words(&["hello", "hi"]).unwrap();
        fake.add_ignored("foobar").unwrap();

        assert!(fake.lookup_word("foobar", None, None).unwrap())
    }

    #[test]
    fn test_fake_repo_lookup_for_extension() {
        let mut fake = FakeRepo::new();
        fake.add_good_words(&["hello", "hi"]).unwrap();
        fake.add_extension("py").unwrap();
        fake.add_ignored_for_extension("defaultdict", "py").unwrap();

        assert!(!fake.lookup_word("defaultdict", None, None).unwrap());
        assert!(fake
            .lookup_word("defaultdict", Some("hello.py"), Some("py"))
            .unwrap());
    }

    #[test]
    fn test_fake_repo_lookup_for_file() {
        let mut fake = FakeRepo::new();
        fake.add_good_words(&["hello", "hi"]).unwrap();
        fake.add_file("poetry.lock").unwrap();
        fake.add_ignored_for_file("abcdef", "poetry.lock").unwrap();

        assert!(!fake.lookup_word("abcdef", None, None).unwrap());
        assert!(fake
            .lookup_word("abcdef", Some("poetry.lock"), Some("lock"))
            .unwrap());
    }

    #[test]
    fn test_fake_interactor_replay_recorderd_answers() {
        let fake_interactor = FakeInteractor::new();
        fake_interactor.push_text("Alice");
        fake_interactor.push_text("blue");
        fake_interactor.push_int(1);
        fake_interactor.push_bool(true);
        fake_interactor.push_text("q");

        let name = fake_interactor.input("What is your name");
        let color = fake_interactor.input("What is your favorite color");
        let index = fake_interactor.select("Cofee or tea?", &["coffee", "tea"]);
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
}
