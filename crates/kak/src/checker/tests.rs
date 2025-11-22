use super::*;

use crate::kak::io::tests::new_fake_io;
use skyspell_core::IgnoreStore;
use skyspell_core::ProjectFile;
use skyspell_core::tests::{FakeDictionary, FakeIO};
use tempfile::TempDir;

pub(crate) type FakeChecker = KakouneChecker<FakeDictionary, FakeIO>;

impl FakeChecker {
    pub(crate) fn get_output(self) -> String {
        self.kakoune_io.get_output()
    }

    pub(crate) fn ensure_path(&self, relative_name: &str) -> ProjectFile {
        let project_path = self.project.path();
        let full_path = project_path.join(relative_name);
        std::fs::write(&full_path, "").unwrap();
        ProjectFile::new(&self.project, &full_path).unwrap()
    }
}

pub(crate) fn new_fake_checker(temp_dir: &TempDir) -> FakeChecker {
    let dictionary = FakeDictionary::new();
    let project_path = temp_dir.path();
    let project = Project::new(project_path).unwrap();
    let mut fake_io = new_fake_io();
    fake_io.set_option("skyspell_project", &project.path_string());
    let state_toml = temp_dir.path().join("state.toml");
    let global_toml = temp_dir.path().join("global.toml");
    let local_toml = temp_dir.path().join("skyspell.toml");
    let ignore_store = IgnoreStore::load(global_toml, local_toml).unwrap();
    KakouneChecker::new(project, dictionary, ignore_store, fake_io, Some(state_toml)).unwrap()
}

fn make_error(word: &str, project_file: &ProjectFile, pos: (usize, usize)) -> SpellingError {
    SpellingError::new(word.to_owned(), pos, project_file)
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
    let error = make_error("foo", &hello_js, (2, 4));
    checker
        .handle_error(&error, &hello_js.name().to_owned())
        .unwrap();
    checker.write_spelling_buffer();
    let actual = checker.get_output();
    let expected = format!(
        "evaluate-commands -draft %{{edit -scratch *spelling*
execute-keys -draft \\% <ret> d i %{{{}/hello.js: 2.5,2.7 foo<ret>}} <esc>}}
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
    let error = make_error("foo", &foo_js, (2, 4));
    checker
        .handle_error(&error, &foo_js.name().to_string())
        .unwrap();

    let error = make_error("bar", &foo_js, (3, 6));
    checker
        .handle_error(&error, &foo_js.name().to_string())
        .unwrap();

    let error = make_error("spam", &bar_js, (1, 5));
    checker
        .handle_error(&error, &bar_js.name().to_string())
        .unwrap();

    let timestamp = 42;
    checker.write_ranges(timestamp);

    let actual = checker.get_output();
    let expected = "\
    set-option %{buffer=foo.js} skyspell_errors 42 2.5+3|SpellingError 3.7+3|SpellingError \n\
    set-option %{buffer=bar.js} skyspell_errors 42 1.6+4|SpellingError \n";
    assert_eq!(actual, expected);
}
