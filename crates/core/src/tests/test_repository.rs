#[macro_export]
macro_rules! test_repository {
    ($repo:ty) => {
        #[allow(unused_imports)]
        use $crate::tests::new_project_path;
        #[allow(unused_imports)]
        use $crate::tests::new_relative_path;

        #[test]
        fn test_insert_ignore() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            repository.ignore("foo").unwrap();

            assert!(repository.is_ignored("foo").unwrap());
            assert!(!repository.is_ignored("bar").unwrap());
        }

        #[test]
        fn test_lookup_extension() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            repository.ignore_for_extension("dict", "py").unwrap();

            assert!(repository.is_ignored_for_extension("dict", "py").unwrap());
        }

        #[test]
        fn test_insert_ignore_ignore_duplicates() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            repository.ignore("foo").unwrap();
            repository.ignore("foo").unwrap();
        }

        #[test]
        fn test_ignored_for_extension_duplicates() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            repository.ignore_for_extension("dict", "py").unwrap();
            repository.ignore_for_extension("dict", "py").unwrap();
        }

        #[test]
        fn test_create_project() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            let temp_dir = tempfile::Builder::new()
                .prefix("test-skyspell")
                .tempdir()
                .unwrap();
            let project = new_project_path(&temp_dir, "project");

            assert!(!repository.project_exists(&project).unwrap());

            repository.new_project(&project).unwrap();
            assert!(repository.project_exists(&project).unwrap());
        }

        #[test]
        fn test_duplicate_projects() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            let temp_dir = tempfile::Builder::new()
                .prefix("test-skyspell")
                .tempdir()
                .unwrap();
            let project = new_project_path(&temp_dir, "project");

            repository.new_project(&project).unwrap();
            assert!(repository.new_project(&project).is_err());
        }

        #[test]
        fn test_remove_project() {
            let mut repository = <$repo>::new_for_tests().unwrap();
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
        }

        #[test]
        fn test_ignored_for_project() {
            let mut repository = <$repo>::new_for_tests().unwrap();
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

            assert!(repository
                .is_ignored_for_project("foo", project_id)
                .unwrap());
            assert!(!repository
                .is_ignored_for_project("foo", other_project_id)
                .unwrap());
        }

        #[test]
        fn test_ignored_for_path() {
            let mut repository = <$repo>::new_for_tests().unwrap();
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
            repository
                .ignore_for_path("foo", project_id, &foo_py)
                .unwrap();

            assert!(repository
                .is_ignored_for_path("foo", project_id, &foo_py)
                .unwrap());
            // Same project, different file
            assert!(!repository
                .is_ignored_for_path("foo", project_id, &foo_rs)
                .unwrap());
            // Same file, different project
            assert!(!repository
                .is_ignored_for_path("foo", other_project_id, &foo_py)
                .unwrap());
        }

        #[test]
        fn test_skip_file_name() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());

            repository.skip_file_name("Cargo.lock").unwrap();
            assert!(repository.is_skipped_file_name("Cargo.lock").unwrap());
        }

        #[test]
        fn test_skip_path() {
            let mut repository = <$repo>::new_for_tests().unwrap();
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
        }

        #[test]
        fn test_remove_ignored_happy() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            repository.ignore("foo").unwrap();

            repository.remove_ignored("foo").unwrap();

            assert!(!repository.is_ignored("foo").unwrap());
        }

        #[test]
        fn test_remove_ignored_when_not_ignored() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            assert!(!repository.is_ignored("foo").unwrap());

            assert!(repository.remove_ignored("foo").is_err());
        }

        #[test]
        fn test_remove_ignored_for_extension_happy() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            repository.ignore_for_extension("foo", "py").unwrap();

            repository
                .remove_ignored_for_extension("foo", "py")
                .unwrap();

            assert!(!repository.is_ignored_for_extension("foo", "py").unwrap());
        }

        #[test]
        fn test_remove_ignored_for_extension_when_not_ignored() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            assert!(!repository.is_ignored_for_extension("foo", "py").unwrap());

            assert!(repository
                .remove_ignored_for_extension("foo", "py")
                .is_err());
        }

        #[test]
        fn test_remove_ignored_for_path_happy() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            let temp_dir = tempfile::Builder::new()
                .prefix("test-skyspell")
                .tempdir()
                .unwrap();
            let project = new_project_path(&temp_dir, "project");
            let project_id = repository.new_project(&project).unwrap();
            let foo_py = new_relative_path(&project, "foo.py");
            repository
                .ignore_for_path("foo", project_id, &foo_py)
                .unwrap();

            repository
                .remove_ignored_for_path("foo", project_id, &foo_py)
                .unwrap();

            assert!(!repository
                .is_ignored_for_path("foo", project_id, &foo_py)
                .unwrap());
        }

        #[test]
        fn test_remove_ignored_for_path_when_not_ignored() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            let temp_dir = tempfile::Builder::new()
                .prefix("test-skyspell")
                .tempdir()
                .unwrap();
            let project = new_project_path(&temp_dir, "project");
            let project_id = repository.new_project(&project).unwrap();
            let foo_py = new_relative_path(&project, "foo.py");

            assert!(!repository
                .is_ignored_for_path("foo", project_id, &foo_py)
                .unwrap());

            assert!(repository
                .remove_ignored_for_path("foo", project_id, &foo_py)
                .is_err());
        }

        #[test]
        fn test_remove_ignored_for_project() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            let temp_dir = tempfile::Builder::new()
                .prefix("test-skyspell")
                .tempdir()
                .unwrap();
            let project = new_project_path(&temp_dir, "project");
            repository.new_project(&project).unwrap();
            let project_id = repository.get_project_id(&project).unwrap();
            repository.ignore_for_project("foo", project_id).unwrap();

            repository
                .remove_ignored_for_project("foo", project_id)
                .unwrap();

            assert!(!repository
                .is_ignored_for_project("foo", project_id)
                .unwrap());
        }

        #[test]
        fn test_unskip_file_name_happy() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            repository.skip_file_name("Cargo.lock").unwrap();

            repository.unskip_file_name("Cargo.lock").unwrap();

            assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());
        }

        #[test]
        fn test_unskip_file_name_not_skipped() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());

            assert!(repository.unskip_file_name("Cargo.lock").is_err())
        }

        #[test]
        fn test_unskip_path_happy() {
            let mut repository = <$repo>::new_for_tests().unwrap();
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
        }

        #[test]
        fn test_unskip_path_not_skipped() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            let temp_dir = tempfile::Builder::new()
                .prefix("test-skyspell")
                .tempdir()
                .unwrap();
            let project = new_project_path(&temp_dir, "project");
            let project_id = repository.new_project(&project).unwrap();
            let foo_py = new_relative_path(&project, "foo.py");
            assert!(!repository.is_skipped_path(project_id, &foo_py).unwrap());

            assert!(repository.unskip_path(project_id, &foo_py).is_err());
        }

        #[test]
        fn test_pop_last_operation_returning_none() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            let actual = repository.pop_last_operation().unwrap();
            assert!(actual.is_none());
        }

        #[test]
        fn test_pop_last_operation_happy() {
            use crate::repository::handler::Ignore;
            use crate::repository::Operation;
            let mut repository = <$repo>::new_for_tests().unwrap();
            let ignore_foo = Operation::Ignore(Ignore {
                word: "foo".to_string(),
            });
            repository.insert_operation(&ignore_foo).unwrap();

            let actual = repository.pop_last_operation().unwrap().unwrap();
            assert_eq!(actual, ignore_foo);
        }
    };
}
