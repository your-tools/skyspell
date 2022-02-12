use std::cell::RefCell;
use std::collections::VecDeque;

use crate::Interactor;

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
