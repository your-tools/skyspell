use super::*;

#[test]
fn test_config() {
    let _config = Config::new();
}

#[test]
fn test_config_set_lang() {
    let mut config = Config::new();
    config.set_lang("en_US");
}

#[test]
fn test_error_when_invalid_lang() {
    let mut config = Config::new();
    config.set_lang("no-such-lang");
    let speller = config.speller();
    assert!(speller.is_err());
}

#[test]
fn test_check_valid_word() {
    let mut config = Config::new();
    config.set_lang("en_US");
    let speller = config.speller().unwrap();
    assert!(speller.check("hello"));
}

#[test]
fn test_check_invalid_word() {
    let mut config = Config::new();
    config.set_lang("en_US");
    let speller = config.speller().unwrap();
    assert!(!speller.check("missstake"));
}

#[test]
fn test_suggest() {
    let mut config = Config::new();
    config.set_lang("en_US");
    let speller = config.speller().unwrap();
    let suggestions = speller.suggest("missstake");
    assert!(suggestions.contains(&"mistake".to_string()));
}
