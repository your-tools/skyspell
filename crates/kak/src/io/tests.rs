use skyspell_core::tests::FakeIO;

use super::*;

pub(crate) type FakeKakouneIO = KakouneIO<FakeIO>;

impl FakeKakouneIO {
    pub(crate) fn get_output(self) -> String {
        self.os_io.get_output()
    }

    pub(crate) fn set_env_var(&mut self, key: &str, value: &str) {
        self.os_io.set_env_var(key, value);
    }

    pub(crate) fn set_option(&mut self, key: &str, value: &str) {
        let key = format!("kak_opt_{key}");
        self.os_io.set_env_var(&key, value);
    }

    pub(crate) fn set_selection(&mut self, text: &str) {
        self.set_env_var("kak_selection", text)
    }

    pub(crate) fn set_timestamp(&mut self, timestamp: usize) {
        self.set_env_var("kak_timestamp", &timestamp.to_string())
    }

    pub(crate) fn set_cursor(&mut self, line: usize, column: usize) {
        self.set_env_var("kak_cursor_line", &line.to_string());
        self.set_env_var("kak_cursor_column", &column.to_string());
    }
}

pub(crate) fn new_fake_io() -> FakeKakouneIO {
    let fake_os_io = FakeIO::new();
    KakouneIO::new(fake_os_io)
}

#[test]
fn test_debug() {
    let kakoune_io = new_fake_io();
    kakoune_io.debug("This is a debug message");
    let actual = kakoune_io.get_output();
    assert_eq!(actual, "echo -debug This is a debug message\n");
}

#[test]
fn test_get_variable_no_such_key() {
    let kakoune_io = new_fake_io();
    let actual = kakoune_io.get_variable("no-such-key");
    assert!(actual.is_err());
}

#[test]
fn test_get_variable_set_in_fake_io() {
    let mut kakoune_io = new_fake_io();
    kakoune_io.set_env_var("my_key", "my_value");
    let actual = kakoune_io.get_variable("my_key").unwrap();
    assert_eq!(actual, "my_value");
}

#[test]
fn test_get_option_no_such_key() {
    let kakoune_io = new_fake_io();
    let actual = kakoune_io.get_option("no-such-key");
    assert!(actual.is_err());
}

#[test]
fn test_get_option_set_in_fake_io() {
    let mut kakoune_io = new_fake_io();
    kakoune_io.set_option("my_opt", "my_value");
    let actual = kakoune_io.get_option("my_opt").unwrap();
    assert_eq!(actual, "my_value");
}

#[test]
fn test_get_timestamp_not_set() {
    let kakoune_io = new_fake_io();
    let actual = kakoune_io.get_timestamp();
    assert!(actual.is_err());
}

#[test]
fn test_get_timestamp_set_in_fake_io() {
    let mut kakoune_io = new_fake_io();
    kakoune_io.set_timestamp(42);
    let actual = kakoune_io.get_timestamp().unwrap();
    assert_eq!(actual, 42);
}

#[test]
fn test_parse_usize_happy() {
    let kakoune_io = new_fake_io();
    let actual = kakoune_io.parse_usize("42").unwrap();
    assert_eq!(actual, 42usize);
}

#[test]
fn test_parse_usize_invalid() {
    let kakoune_io = new_fake_io();
    let actual = kakoune_io.parse_usize("-3");
    assert!(actual.is_err());
}

#[test]
fn test_get_cursor_set_in_fake() {
    let mut kakoune_io = new_fake_io();
    kakoune_io.set_cursor(3, 45);
    let actual = kakoune_io.get_cursor().unwrap();
    assert_eq!(actual, (3, 45));
}

#[test]
fn test_parse_range_spec_empty() {
    let kakoune_io = new_fake_io();
    let actual = kakoune_io.parse_range_spec("0").unwrap();
    assert!(actual.is_empty());
}

#[test]
fn test_parse_range_spec_not_empty() {
    let kakoune_io = new_fake_io();
    let actual = kakoune_io
        .parse_range_spec("42 1.4,1.13|SpellingError 1.15,1.23|SpellingError")
        .unwrap();
    assert_eq!(actual, vec![(1, 4, 13), (1, 15, 23)]);
}

#[test]
fn test_get_selection_missing_key() {
    let kakoune_io = new_fake_io();
    let err = kakoune_io.get_selection().unwrap_err();
    assert_eq!(err.to_string(), "No such key: kak_selection");
}

#[test]
fn test_get_selection_set_in_fake() {
    let mut kakoune_io = new_fake_io();
    kakoune_io.set_selection("selected text");
    let actual = kakoune_io.get_selection().unwrap();
    assert_eq!(actual, "selected text");
}

#[test]
fn test_get_previous_selection_between_two_selections_other_line() {
    let kakoune_io = new_fake_io();
    let pos = (1, 21);
    let ranges = [(1, 12, 19), (2, 19, 27)];
    let actual = kakoune_io.get_previous_selection(pos, &ranges).unwrap();
    assert_eq!(actual, &(1, 12, 19));
}

#[test]
fn test_get_previous_selection_from_next_line() {
    let kakoune_io = new_fake_io();
    let pos = (2, 1);
    let ranges = [(1, 12, 19)];
    let actual = kakoune_io.get_previous_selection(pos, &ranges).unwrap();
    assert_eq!(actual, &(1, 12, 19));
}

#[test]
fn test_get_previous_selection_wraps() {
    let kakoune_io = new_fake_io();
    let pos = (1, 1);
    let ranges = [(2, 3, 5), (3, 12, 19)];
    let actual = kakoune_io.get_previous_selection(pos, &ranges).unwrap();
    assert_eq!(actual, &(3, 12, 19));
}

#[test]
fn test_get_previous_selection_between_two_selections_same_line() {
    let kakoune_io = new_fake_io();
    let pos = (1, 15);
    let ranges = [(1, 12, 13), (1, 19, 21)];
    let actual = kakoune_io.get_previous_selection(pos, &ranges).unwrap();
    assert_eq!(actual, &(1, 12, 13));
}

#[test]
fn test_get_next_selection_from_previous_line() {
    let kakoune_io = new_fake_io();
    let pos = (1, 1);
    let ranges = [(2, 12, 19)];
    let actual = kakoune_io.get_next_selection(pos, &ranges).unwrap();
    assert_eq!(actual, &(2, 12, 19));
}

#[test]
fn test_get_next_selection_wraps() {
    let kakoune_io = new_fake_io();
    let pos = (4, 1);
    let ranges = [(2, 3, 5), (3, 12, 19)];
    let actual = kakoune_io.get_next_selection(pos, &ranges).unwrap();
    assert_eq!(actual, &(2, 3, 5));
}

#[test]
fn test_get_next_selection_between_two_selections_same_line() {
    let kakoune_io = new_fake_io();
    let pos = (1, 15);
    let ranges = [(1, 12, 13), (1, 19, 21)];
    let actual = kakoune_io.get_next_selection(pos, &ranges).unwrap();
    assert_eq!(actual, &(1, 19, 21));
}
