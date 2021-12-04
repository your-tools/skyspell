use skyspell_core::Dictionary;

use super::*;

#[test]
fn test_new() {
    AspellDictionary::new("en_US").unwrap();
}

#[test]
fn test_check_valid_word() {
    let dict = AspellDictionary::new("en_US").unwrap();
    assert!(!dict.check("missstake").unwrap());
}

#[test]
fn test_check_invalid_word() {
    let dict = AspellDictionary::new("en_US").unwrap();
    assert!(!dict.check("missstake").unwrap());
}

#[test]
fn test_suggest() {
    let dict = AspellDictionary::new("en_US").unwrap();
    let suggestions = dict.suggest("missstake");
    assert!(suggestions.contains(&"mistake".to_string()));
}
