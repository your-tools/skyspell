use crate::FakeRepository;
use crate::{new_project_path, new_relative_path};
use crate::{IgnoreStore, Repository};

#[test]
fn test_should_ignore_when_in_global_list() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let relative_path = new_relative_path(&project, "foo");
    let mut repository = FakeRepository::new();

    repository.ignore("foo").unwrap();
    let project_id = repository.new_project(&project).unwrap();

    assert!(repository
        .should_ignore("foo", project_id, &relative_path)
        .unwrap());
}

#[test]
fn test_should_ignore_when_in_list_for_extension() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let foo_py = new_relative_path(&project, "foo.py");
    let foo_rs = new_relative_path(&project, "foo.rs");

    let mut repository = FakeRepository::new();
    let project_id = repository.new_project(&project).unwrap();
    repository.ignore_for_extension("foo", "py").unwrap();

    assert!(repository
        .should_ignore("foo", project_id, &foo_py)
        .unwrap());

    assert!(!repository
        .should_ignore("foo", project_id, &foo_rs)
        .unwrap());
}

#[test]
fn test_should_ignore_when_in_project_list() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project_1 = new_project_path(&temp_dir, "project1");
    let foo_txt = new_relative_path(&project_1, "foo.txt");
    let project_2 = new_project_path(&temp_dir, "project2");
    let mut repository = FakeRepository::new();
    let project_id_1 = repository.new_project(&project_1).unwrap();
    let project_id_2 = repository.new_project(&project_2).unwrap();

    repository.ignore_for_project("foo", project_id_1).unwrap();

    assert!(repository
        .should_ignore("foo", project_id_1, &foo_txt)
        .unwrap());
    assert!(!repository
        .should_ignore("foo", project_id_2, &foo_txt)
        .unwrap());
}
