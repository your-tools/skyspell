use diesel::dsl::count_star;
use diesel::prelude::*;
use skyspell_core::repository::handler::Ignore;
use skyspell_core::repository::Operation;
use skyspell_core::Repository;
use skyspell_sql::schema::operations;
use skyspell_sql::SQLRepository;
use skyspell_tests::FakeRepository;
use skyspell_tests::{new_project_path, new_relative_path};

use paste::paste;

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
            let repository = SQLRepository::in_memory().unwrap();
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

make_tests!(pop_last_operation_happy, (repository) => {
    let ignore_foo = Operation::Ignore(Ignore { word: "foo".to_string() });
    repository.insert_operation(&ignore_foo).unwrap();

    let actual = repository.pop_last_operation().unwrap().unwrap();
    assert_eq!(actual, ignore_foo);
});

#[test]
fn test_delete_old_operations_when_more_than_100_operations_are_stored() {
    let mut sql_repository = SQLRepository::in_memory().unwrap();
    let values: Vec<_> = (1..=103)
        .map(|i| {
            let word = format!("foo-{}", i);
            let operation = Operation::Ignore(Ignore { word });
            let json = serde_json::to_string(&operation).unwrap();
            (
                operations::json.eq(json),
                operations::timestamp.eq(i + 10_000),
            )
        })
        .collect();
    diesel::insert_into(operations::table)
        .values(values)
        .execute(&sql_repository.connection)
        .unwrap();

    let last = sql_repository.pop_last_operation().unwrap();
    assert!(last.is_some());

    let actual_count: i64 = operations::table
        .select(count_star())
        .first(&sql_repository.connection)
        .unwrap();

    assert_eq!(actual_count, 101);
}

#[test]
fn test_keep_old_operations_when_less_than_100_operations_are_stored() {
    let mut sql_repository = SQLRepository::in_memory().unwrap();
    let values: Vec<_> = (1..=50)
        .map(|i| {
            let word = format!("foo-{}", i);
            let operation = Operation::Ignore(Ignore { word });
            let json = serde_json::to_string(&operation).unwrap();
            (
                operations::json.eq(json),
                operations::timestamp.eq(i + 10_000),
            )
        })
        .collect();
    diesel::insert_into(operations::table)
        .values(values)
        .execute(&sql_repository.connection)
        .unwrap();

    let last = sql_repository.pop_last_operation().unwrap();
    assert!(last.is_some());

    let actual_count: i64 = operations::table
        .select(count_star())
        .first(&sql_repository.connection)
        .unwrap();

    assert_eq!(actual_count, 49);
}
