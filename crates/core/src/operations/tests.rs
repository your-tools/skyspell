use std::path::PathBuf;

use crate::{
    Project,
    tests::{get_empty_store, get_test_dir},
};

use super::*;

#[test]
fn test_undo_global_ignore() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let mut operation = Operation::new_ignore("foo");
    operation.execute(&mut store).unwrap();
    assert!(store.is_ignored("foo"));

    operation.undo(&mut store).unwrap();

    assert!(!store.is_ignored("foo"));
}

#[test]
fn test_undo_ignore_for_extension() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let mut operation = Operation::new_ignore_for_extension("foo", "py");
    operation.execute(&mut store).unwrap();
    assert!(store.is_ignored_for_extension("foo", "py"));

    operation.undo(&mut store).unwrap();

    assert!(!store.is_ignored_for_extension("foo", "py"));
}

#[test]
fn test_undo_ignore_for_path() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let project = Project::new(temp_dir.path()).unwrap();
    let foo_py = PathBuf::from("foo.py");
    let foo_py = ProjectFile::new(&project, &foo_py).unwrap();
    let mut operation = Operation::new_ignore_for_path("foo", &foo_py);
    operation.execute(&mut store).unwrap();
    assert!(store.is_ignored_for_path("foo", &foo_py));

    operation.undo(&mut store).unwrap();

    assert!(!store.is_ignored_for_path("foo", &foo_py));
}

#[test]
fn test_undo_ignore_for_project() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let mut operation = Operation::new_ignore_for_project("foo");
    operation.execute(&mut store).unwrap();
    assert!(store.is_ignored_for_project("foo"));

    operation.undo(&mut store).unwrap();

    assert!(!store.is_ignored_for_project("foo"));
}

#[test]
fn test_undo_ignore_for_lang() {
    let temp_dir = get_test_dir();
    let mut store = get_empty_store(&temp_dir);
    let mut operation = Operation::new_ignore_for_lang("foo", "en");
    operation.execute(&mut store).unwrap();
    assert!(store.is_ignored_for_lang("foo", "en"));

    operation.undo(&mut store).unwrap();

    assert!(!store.is_ignored_for_lang("foo", "en"));
}
