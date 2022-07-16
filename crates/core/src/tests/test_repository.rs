#[macro_export]
macro_rules! test_repository {
    ($repo:ty) => {
        #[allow(unused_imports)]
        use $crate::tests::new_project_path;
        #[allow(unused_imports)]
        use $crate::tests::new_relative_path;

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
        fn test_pop_last_operation_returning_none() {
            let mut repository = <$repo>::new_for_tests().unwrap();
            let actual = repository.pop_last_operation().unwrap();
            assert!(actual.is_none());
        }

        #[test]
        fn test_pop_last_operation_happy() {
            use $crate::undo::Ignore;
            use $crate::Operation;
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
