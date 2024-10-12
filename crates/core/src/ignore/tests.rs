use tempfile::TempDir;

use crate::{
    tests::{create_store, get_test_dir},
    RelativePath,
};

use super::*;

#[test]
fn test_add_for_extension_writes_in_global_toml() {
    let temp_dir = get_test_dir();
    let mut store = create_store(
        &temp_dir,
        r#"
        global = ["one"]

        [extensions]
        rs = ["fn"]
        "#,
        r#"
        patterns = ["Cargo.lock"]
        "#,
    );

    store.ignore_for_extension("impl", "rs").unwrap();

    let global_toml = temp_dir.path().join("global.toml");
    let actual: GlobalIgnore = load(&global_toml).unwrap();
    assert_eq!(
        actual.extensions["rs"].iter().collect::<Vec<_>>(),
        vec!["fn", "impl"]
    );
}

fn get_empty_store(temp_dir: &TempDir) -> IgnoreStore {
    create_store(temp_dir, "", "")
}

#[test]
fn test_insert_ignore() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);

    store.ignore("foo").unwrap();

    assert!(store.is_ignored("foo"));
    assert!(!store.is_ignored("bar"));
}

#[test]
fn test_lookup_extension() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);

    store.ignore_for_extension("dict", "py").unwrap();

    assert!(store.is_ignored_for_extension("dict", "py"));
}

#[test]
fn test_insert_ignore_ignore_duplicates() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);

    store.ignore("foo").unwrap();
    store.ignore("foo").unwrap();
}

#[test]
fn test_ignored_for_project() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);

    store.ignore_for_project("foo").unwrap();

    assert!(store.is_ignored_for_project("foo"))
}

#[test]
fn test_ignored_for_path() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);

    let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));
    let foo_rs = RelativePath::from_path_unchecked(PathBuf::from("foo.rs"));

    store.ignore_for_path("foo", &foo_py).unwrap();

    assert!(store.is_ignored_for_path("foo", &foo_py));
    assert!(!store.is_ignored_for_path("foo", &foo_rs));
}
#[test]
fn test_remove_ignored_happy() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    store.ignore("foo").unwrap();

    store.remove_ignored("foo").unwrap();

    assert!(!store.is_ignored("foo"));
}

#[test]
fn test_remove_ignored_when_not_ignored() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);

    assert!(store.remove_ignored("foo").is_err());
}

#[test]
fn test_remove_ignored_for_extension_happy() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    store.ignore_for_extension("foo", "py").unwrap();

    store.remove_ignored_for_extension("foo", "py").unwrap();

    assert!(!store.is_ignored_for_extension("foo", "py"));
}

#[test]
fn test_remove_ignored_for_extension_when_not_ignored() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);

    assert!(store.remove_ignored_for_extension("foo", "py").is_err());
}

#[test]
fn test_remove_ignored_for_path_happy() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));

    store.ignore_for_path("foo", &foo_py).unwrap();

    store.remove_ignored_for_path("foo", &foo_py).unwrap();

    assert!(!store.is_ignored_for_path("foo", &foo_py));
}

#[test]
fn test_remove_ignored_for_path_when_not_ignored() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));

    assert!(store.remove_ignored_for_path("foo", &foo_py).is_err());
}

#[test]
fn test_remove_ignored_for_project_happy() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);

    store.ignore_for_project("foo").unwrap();

    store.remove_ignored_for_project("foo").unwrap();

    assert!(!store.is_ignored_for_project("foo"));
}

#[test]
fn test_remove_ignored_for_project_when_not_ignored() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);

    store.remove_ignored_for_project("foo").unwrap_err();
}

fn relative_path(path: &str) -> RelativePath {
    RelativePath::from_path_unchecked(path.into())
}

#[test]
fn test_should_ignore_global() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let foo_py = relative_path("foo.py");

    store.ignore("foo").unwrap();

    assert!(store.should_ignore("foo", &foo_py, "en_US"));
}

#[test]
fn test_should_ignore_extension() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let foo_py = relative_path("foo.py");

    store.ignore_for_extension("foo", "py").unwrap();

    assert!(store.should_ignore("foo", &foo_py, "en_US"));
}

#[test]
fn test_should_ignore_path() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let foo_py = relative_path("foo.py");

    store.ignore_for_path("foo", &foo_py).unwrap();

    assert!(store.should_ignore("foo", &foo_py, "en_US"));
}

#[test]
fn test_should_ignore_project() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let foo_py = relative_path("foo.py");

    store.ignore_for_project("foo").unwrap();

    assert!(store.should_ignore("foo", &foo_py, "en_US"));
}

#[test]
fn test_should_ignore_lang() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let foo_py = relative_path("foo.py");

    store.ignore_for_lang("foo", "en_US").unwrap();

    assert!(store.should_ignore("foo", &foo_py, "en_US"));
}
