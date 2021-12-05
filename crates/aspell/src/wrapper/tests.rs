use super::*;

#[test]
fn test_config() {
    let _config = Config::new();
}

#[test]
fn test_config_set_lang_ok() {
    let mut config = Config::new();
    config.set_lang("en_US").unwrap();
}

#[test]
fn test_config_set_lang_null_byte() {
    let mut config = Config::new();
    let lang = "foo\0bar";
    assert!(config.set_lang(lang).is_err());
}

#[test]
fn test_error_when_invalid_lang() {
    let mut config = Config::new();
    config.set_lang("no-such-lang").unwrap();
    let speller = config.speller();
    assert!(speller.is_err());
}

#[test]
fn test_check_valid_word() {
    let mut config = Config::new();
    config.set_lang("en_US").unwrap();
    let speller = config.speller().unwrap();
    assert!(speller.check("hello").unwrap());
}

#[test]
fn test_check_invalid_word() {
    let mut config = Config::new();
    config.set_lang("en_US").unwrap();
    let speller = config.speller().unwrap();
    assert!(!speller.check("missstake").unwrap());
}

#[test]
fn test_suggest() {
    let mut config = Config::new();
    config.set_lang("en_US").unwrap();
    let speller = config.speller().unwrap();
    let suggestions = speller.suggest("missstake");
    assert!(suggestions.contains(&"mistake".to_string()));
}
