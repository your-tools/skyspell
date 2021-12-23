use skyspell_core::{IgnoreStore, Repository};
use skyspell_tests::FakeRepository;
use skyspell_tests::{new_project_path, new_relative_path};

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

#[test]
fn test_should_skip_when_in_skipped_file_names_list() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let cargo_lock = new_relative_path(&project, "Cargo.lock");
    let poetry_lock = new_relative_path(&project, "poetry.lock");

    let mut repository = FakeRepository::new();
    repository.new_project(&project).unwrap();
    repository.skip_file_name("Cargo.lock").unwrap();
    let project_id = repository.get_project_id(&project).unwrap();

    assert!(repository.should_skip(project_id, &cargo_lock).unwrap());
    assert!(!repository.should_skip(project_id, &poetry_lock).unwrap());
}

#[test]
fn test_should_skip_when_in_skipped_paths() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project_1 = new_project_path(&temp_dir, "project1");
    let foo_txt = new_relative_path(&project_1, "foo.txt");
    let other = new_relative_path(&project_1, "other");
    let project_2 = new_project_path(&temp_dir, "project2");

    let mut repository = FakeRepository::new();
    let project_id_1 = repository.new_project(&project_1).unwrap();
    let project_id_2 = repository.new_project(&project_2).unwrap();

    repository.skip_path(project_id_1, &foo_txt).unwrap();

    assert!(repository.should_skip(project_id_1, &foo_txt).unwrap());

    // Same project, other path
    assert!(!repository.should_skip(project_id_1, &other).unwrap());

    // Same file, other project
    assert!(!repository.should_skip(project_id_2, &foo_txt).unwrap());
}