use std::collections::HashMap;

use anyhow::Result;
use crate::Dictionary;

#[derive(Default)]
pub struct FakeDictionary {
    known: Vec<String>,
    suggestions: HashMap<String, Vec<String>>,
}

impl FakeDictionary {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_known(&mut self, word: &str) {
        self.known.push(word.to_string());
    }

    pub fn add_suggestions(&mut self, error: &str, suggestions: &[String]) {
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

    fn lang(&self) -> &str {
        "en_US"
    }

    fn provider(&self) -> &str {
        "fake"
    }
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
