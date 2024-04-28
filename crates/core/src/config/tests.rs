use super::*;
use std::path::PathBuf;

#[test]
fn test_empty_config_is_valid() {
    Config::parse("").unwrap();
}

#[test]
fn test_insert_ignore() {
    let mut config = Config::empty();
    config.ignore("foo").unwrap();

    assert!(config.is_ignored("foo").unwrap());
    assert!(!config.is_ignored("bar").unwrap());
}

#[test]
fn test_lookup_extension() {
    let mut config = Config::empty();
    config.ignore_for_extension("dict", "py").unwrap();

    assert!(config.is_ignored_for_extension("dict", "py").unwrap());
}

#[test]
fn test_insert_ignore_ignore_duplicates() {
    let mut config = Config::empty();
    config.ignore("foo").unwrap();
    config.ignore("foo").unwrap();
}

#[test]
fn test_ignored_for_extension_duplicates() {
    let mut config = Config::empty();
    config.ignore_for_extension("dict", "py").unwrap();
    config.ignore_for_extension("dict", "py").unwrap();
    let plop = toml_edit::ser::to_string_pretty(&config.inner).unwrap();
    println!("{plop}");
}

#[test]
fn test_ignored_for_project() {
    let mut config = Config::empty();
    config.ignore_for_project("foo").unwrap();

    assert!(config.is_ignored_for_project("foo").unwrap())
}

#[test]
fn test_ignored_for_path() {
    let mut config = Config::empty();
    let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));
    let foo_rs = RelativePath::from_path_unchecked(PathBuf::from("foo.rs"));

    config.ignore_for_path("foo", &foo_py).unwrap();

    assert!(config.is_ignored_for_path("foo", &foo_py).unwrap());
    assert!(!config.is_ignored_for_path("foo", &foo_rs).unwrap());
}

#[test]
fn test_remove_ignored_happy() {
    let mut config = Config::empty();
    config.ignore("foo").unwrap();

    config.remove_ignored("foo").unwrap();

    assert!(!config.is_ignored("foo").unwrap());
}

#[test]
fn test_remove_ignored_when_not_ignored() {
    let mut config = Config::empty();
    assert!(!config.is_ignored("foo").unwrap());

    assert!(config.remove_ignored("foo").is_err());
}

#[test]
fn test_remove_ignored_for_extension_happy() {
    let mut config = Config::empty();
    config.ignore_for_extension("foo", "py").unwrap();

    config.remove_ignored_for_extension("foo", "py").unwrap();

    assert!(!config.is_ignored_for_extension("foo", "py").unwrap());
}

#[test]
fn test_remove_ignored_for_extension_when_not_ignored() {
    let mut config = Config::empty();
    assert!(!config.is_ignored_for_extension("foo", "py").unwrap());

    assert!(config.remove_ignored_for_extension("foo", "py").is_err());
}

#[test]
fn test_remove_ignored_for_path_happy() {
    let mut config = Config::empty();
    let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));

    config.ignore_for_path("foo", &foo_py).unwrap();

    config.remove_ignored_for_path("foo", &foo_py).unwrap();

    assert!(!config.is_ignored_for_path("foo", &foo_py).unwrap());
}

#[test]
fn test_remove_ignored_for_path_when_not_ignored() {
    let mut config = Config::empty();
    let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));

    assert!(!config.is_ignored_for_path("foo", &foo_py).unwrap());

    assert!(config.remove_ignored_for_path("foo", &foo_py).is_err());
}

#[test]
fn test_remove_ignored_for_project() {
    let mut config = Config::empty();
    config.ignore_for_project("foo").unwrap();

    config.remove_ignored_for_project("foo").unwrap();

    assert!(!config.is_ignored_for_project("foo").unwrap());
}
