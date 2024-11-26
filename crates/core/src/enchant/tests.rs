use super::*;

#[test]
fn test_check_invalid_word() {
    let dict = EnchantDictionary::new("en_US").unwrap();
    assert!(!dict.check("missstake").unwrap());
}

#[test]
fn test_check_valid_word() {
    let dict = EnchantDictionary::new("en_US").unwrap();
    assert!(dict.check("valid").unwrap());
}
