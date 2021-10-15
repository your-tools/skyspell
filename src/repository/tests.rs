use paste::paste;
use tempfile::TempDir;

use crate::sql::SQLRepository;
use crate::tests::FakeRepository;
use crate::Ignore;

use super::*;

fn new_project_path(temp_dir: &TempDir, name: &'static str) -> ProjectPath {
    let temp_path = temp_dir.path();
    let project_path = temp_path.join(name);
    std::fs::create_dir(&project_path).unwrap();
    ProjectPath::new(&project_path).unwrap()
}

fn new_relative_path(project_path: &ProjectPath, name: &'static str) -> RelativePath {
    let rel_path = project_path.as_ref().join(name);
    std::fs::write(&rel_path, "").unwrap();
    RelativePath::new(project_path, &rel_path).unwrap()
}

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

// Given an identifier and a block, generate a test
// for each implementation of the Repository trait
// (SQLRepository and FakeRepository)
macro_rules! make_tests {
    ($name:ident, ($repository:ident) => $test:block) => {
        paste! {
        fn $name(mut $repository: impl Repository) {
            $test
        }

        #[test]
        fn [<test_sql_repository_ $name>]() {
            let repository = SQLRepository::in_memory();
            $name(repository)
        }

        #[test]
        fn [<test_fake_repository_ $name>]() {
            let repository = FakeRepository::new();
            $name(repository)
        }
        } // end paste
    };
}

make_tests!(insert_ignore, (repository) => {
    repository.ignore("foo").unwrap();

    assert!(repository.is_ignored("foo").unwrap());
    assert!(!repository.is_ignored("bar").unwrap());
});

make_tests!(lookup_extension, (repository) => {
    repository.ignore_for_extension("dict", "py").unwrap();

    assert!(repository.is_ignored_for_extension("dict", "py").unwrap());
});

make_tests!(insert_ignore_ignore_duplicates, (repository) => {
    repository.ignore("foo").unwrap();
    repository.ignore("foo").unwrap();
});

make_tests!(ignored_for_extension_duplicates, (repository) => {
    repository.ignore_for_extension("dict", "py").unwrap();
    repository.ignore_for_extension("dict", "py").unwrap();
});

make_tests!(create_project, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");

    assert!(!repository.project_exists(&project).unwrap());

    repository.new_project(&project).unwrap();
    assert!(repository.project_exists(&project).unwrap());
});

make_tests!(duplicate_projects, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");

    repository.new_project(&project).unwrap();
    assert!(repository.new_project(&project).is_err());
});

make_tests!(remove_project, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project1 = new_project_path(&temp_dir, "project1");
    let project2 = new_project_path(&temp_dir, "project2");
    let project3 = new_project_path(&temp_dir, "project3");
    repository.new_project(&project1).unwrap();
    let project2_id = repository.new_project(&project2).unwrap();
    repository.new_project(&project3).unwrap();

    repository.remove_project(project2_id).unwrap();

    assert!(!repository.project_exists(&project2).unwrap());
});

make_tests!(ignored_for_project, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let other_project = new_project_path(&temp_dir, "other");

    repository.new_project(&project).unwrap();
    repository.new_project(&other_project).unwrap();

    let project_id = repository.get_project_id(&project).unwrap();
    let other_project_id = repository.get_project_id(&other_project).unwrap();
    repository.ignore_for_project("foo", project_id).unwrap();

    assert!(repository.is_ignored_for_project("foo", project_id).unwrap());
    assert!(!repository.is_ignored_for_project("foo", other_project_id).unwrap());
});

make_tests!(ignored_for_path, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let foo_py = new_relative_path(&project, "foo.py");
    let foo_rs = new_relative_path(&project, "foo.rs");
    let other_project = new_project_path(&temp_dir, "other");
    repository.new_project(&project).unwrap();
    repository.new_project(&other_project).unwrap();

    let project_id = repository.get_project_id(&project).unwrap();
    let other_project_id = repository.get_project_id(&other_project).unwrap();
    repository.ignore_for_path("foo", project_id, &foo_py).unwrap();

    assert!(repository.is_ignored_for_path("foo", project_id, &foo_py).unwrap());
    // Same project, different file
    assert!(!repository.is_ignored_for_path("foo", project_id, &foo_rs).unwrap());
    // Same file, different project
    assert!(!repository.is_ignored_for_path("foo", other_project_id, &foo_py).unwrap());
});

make_tests!(skip_file_name, (repository) => {
    assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());

    repository.skip_file_name("Cargo.lock").unwrap();
    assert!(repository.is_skipped_file_name("Cargo.lock").unwrap());
});

make_tests!(skip_path, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let test_txt = new_relative_path(&project, "test.txt");

    let project_id = repository.new_project(&project).unwrap();
    assert!(!repository.is_skipped_path(project_id, &test_txt).unwrap());

    repository.skip_path(project_id, &test_txt).unwrap();

    assert!(repository.is_skipped_path(project_id, &test_txt).unwrap());
});

make_tests!(remove_ignored_happy, (repository) => {
    repository.ignore("foo").unwrap();

    repository.remove_ignored("foo").unwrap();

    assert!(!repository.is_ignored("foo").unwrap());
});

make_tests!(remove_ignored_when_not_ignored, (repository) => {
    assert!(!repository.is_ignored("foo").unwrap());

    assert!(repository.remove_ignored("foo").is_err());

});

make_tests!(remove_ignored_for_extension_happy, (repository) => {
    repository.ignore_for_extension("foo", "py").unwrap();

    repository.remove_ignored_for_extension("foo", "py").unwrap();

    assert!(!repository.is_ignored_for_extension("foo", "py").unwrap());

});

make_tests!(remove_ignored_for_extension_when_not_ignored, (repository) => {
    assert!(!repository.is_ignored_for_extension("foo", "py").unwrap());

    assert!(repository.remove_ignored_for_extension("foo", "py").is_err());
});

make_tests!(remove_ignored_for_path_happy, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let project_id = repository.new_project(&project).unwrap();
    let foo_py = new_relative_path(&project, "foo.py");
    repository.ignore_for_path("foo", project_id, &foo_py).unwrap();

    repository.remove_ignored_for_path("foo", project_id, &foo_py).unwrap();

    assert!(!repository.is_ignored_for_path("foo", project_id, &foo_py).unwrap());
});

make_tests!(remove_ignored_for_path_when_not_ignored, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let project_id = repository.new_project(&project).unwrap();
    let foo_py = new_relative_path(&project, "foo.py");

    assert!(!repository.is_ignored_for_path("foo", project_id, &foo_py).unwrap());

    assert!(repository.remove_ignored_for_path("foo", project_id, &foo_py).is_err());
});

make_tests!(remove_ignored_for_project, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    repository.new_project(&project).unwrap();
    let project_id = repository.get_project_id(&project).unwrap();
    repository.ignore_for_project("foo", project_id).unwrap();

    repository.remove_ignored_for_project("foo", project_id).unwrap();

    assert!(!repository.is_ignored_for_project("foo", project_id).unwrap());
});

make_tests!(unskip_file_name_happy, (repository) => {
    repository.skip_file_name("Cargo.lock").unwrap();

    repository.unskip_file_name("Cargo.lock").unwrap();

    assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());
});

make_tests!(unskip_file_name_not_skipped, (repository) => {
    assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());

    assert!(repository.unskip_file_name("Cargo.lock").is_err())

});

make_tests!(unskip_path_happy, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let project_id = repository.new_project(&project).unwrap();
    let foo_py = new_relative_path(&project, "foo.py");
    repository.skip_path(project_id, &foo_py).unwrap();

    repository.unskip_path(project_id, &foo_py).unwrap();

    assert!(!repository.is_skipped_path(project_id, &foo_py).unwrap());
});

make_tests!(unskip_path_not_skipped, (repository) => {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let project = new_project_path(&temp_dir, "project");
    let project_id = repository.new_project(&project).unwrap();
    let foo_py = new_relative_path(&project, "foo.py");
    assert!(!repository.is_skipped_path(project_id, &foo_py).unwrap());

    assert!(repository.unskip_path(project_id, &foo_py).is_err());

});

make_tests!(pop_last_operation_returning_none, (repository) => {
    let actual = repository.pop_last_operation().unwrap();
    assert!(actual.is_none());
});

use crate::repository::handler::Ignore as IgnoreOperation;
make_tests!(pop_last_operation_happy, (repository) => {
    let ignore_foo = Operation::Ignore(IgnoreOperation { word: "foo".to_string() });
    repository.insert_operation(&ignore_foo).unwrap();

    let actual = repository.pop_last_operation().unwrap().unwrap();
    assert_eq!(actual, ignore_foo);
});
