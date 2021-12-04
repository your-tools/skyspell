// Note: these tests will fail if for some reason 'missstake' is in
// the personal dict, or if no Enchant provider for the US English dictionary is found,
// and there's no good way to know :(

use super::*;

#[test]
fn test_check() {
    let dict = EnchantDictionary::new("en_US").unwrap();
    assert!(!dict.check("missstake").unwrap());
}

#[test]
fn test_suggest() {
    let dict = EnchantDictionary::new("en_US").unwrap();
    assert!(!dict.check("missstake").unwrap());
}