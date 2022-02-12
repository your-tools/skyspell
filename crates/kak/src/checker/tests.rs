use super::*;

use tempfile::TempDir;

use skyspell_core::tests::{FakeDictionary, FakeIO, FakeRepository};
use skyspell_core::{ProjectPath, RelativePath};

use crate::io::tests::new_fake_io;

pub(crate) type FakeChecker = KakouneChecker<FakeDictionary, FakeRepository, FakeIO>;

impl FakeChecker {
    pub(crate) fn get_output(self) -> String {
        self.kakoune_io.get_output()
    }

    pub fn add_known(&mut self, word: &str) {
        self.dictionary.add_known(word);
    }

    pub fn add_suggestions(&mut self, error: &str, suggestions: &[String]) {
        self.dictionary.add_suggestions(error, suggestions);
    }

    pub(crate) fn ensure_path(&self, relative_name: &str) -> RelativePath {
        let project_path = self.project.path();
        let full_path = project_path.as_ref().join(relative_name);
        std::fs::write(&full_path, "").unwrap();
        RelativePath::new(project_path, &full_path).unwrap()
    }
}

pub(crate) fn new_fake_checker(temp_dir: &TempDir) -> FakeChecker {
    let project = ProjectPath::new(temp_dir.path()).unwrap();
    let dictionary = FakeDictionary::new();
    let repository = FakeRepository::new();
    let mut fake_io = new_fake_io();
    fake_io.set_option("skyspell_project", &project.as_str());
    KakouneChecker::new(project, dictionary, repository, fake_io).unwrap()
}

#[test]
fn test_write_errors_in_spelling_buffer() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut checker = new_fake_checker(&temp_dir);
    let hello_js = checker.ensure_path("hello.js");
    checker.ensure_path("hello.js");
    checker
        .handle_error("foo", &hello_js, &(hello_js.to_string(), 2, 4))
        .unwrap();
    checker.write_spelling_buffer();
    let actual = checker.get_output();
    let expected = format!(
        "edit -scratch *spelling*
execute-keys \\% <ret> d i %{{{}/hello.js: 2.5,2.7 foo<ret>}} <esc> gg
",
        temp_dir.path().display()
    );
    assert_eq!(actual, expected);
}

#[test]
fn test_write_errors_as_buffer_options() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut checker = new_fake_checker(&temp_dir);
    let foo_js = checker.ensure_path("foo.js");
    let bar_js = checker.ensure_path("bar.js");
    checker
        .handle_error("foo", &foo_js, &(foo_js.to_string(), 2, 4))
        .unwrap();

    checker
        .handle_error("bar", &foo_js, &(foo_js.to_string(), 3, 6))
        .unwrap();

    checker
        .handle_error("spam", &bar_js, &(bar_js.to_string(), 1, 5))
        .unwrap();

    let timestamp = 42;
    checker.write_ranges(timestamp);

    let actual = checker.get_output();
    let expected = "\
    set-option buffer=foo.js spell_errors 42 2.5+3|SpellingError 3.7+3|SpellingError \n\
    set-option buffer=bar.js spell_errors 42 1.6+4|SpellingError \n";
    assert_eq!(actual, expected);
}
