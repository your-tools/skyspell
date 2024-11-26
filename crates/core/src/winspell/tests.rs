use super::*;
use std::sync::Once;

use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};

#[test]
fn test_check_invalid_word() {
    let dict = SpellClient::new("en").unwrap();
    assert!(!dict.check("missstake").unwrap());
}

#[test]
fn tetst_check_valid_word() {
    let dict = SpellClient::new("en").unwrap();
    assert!(dict.check("hello").unwrap());
}
