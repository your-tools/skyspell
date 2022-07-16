#[macro_export]
macro_rules! test_ignore_store {
    ($repo:ty) => {
        #[allow(unused_imports)]
        use std::path::PathBuf;
        #[allow(unused_imports)]
        use $crate::RelativePath;

        #[test]
        fn test_insert_ignore() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            ignore_store.ignore("foo").unwrap();

            assert!(ignore_store.is_ignored("foo").unwrap());
            assert!(!ignore_store.is_ignored("bar").unwrap());
        }

        #[test]
        fn test_lookup_extension() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            ignore_store.ignore_for_extension("dict", "py").unwrap();

            assert!(ignore_store.is_ignored_for_extension("dict", "py").unwrap());
        }

        #[test]
        fn test_insert_ignore_ignore_duplicates() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            ignore_store.ignore("foo").unwrap();
            ignore_store.ignore("foo").unwrap();
        }

        #[test]
        fn test_ignored_for_extension_duplicates() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            ignore_store.ignore_for_extension("dict", "py").unwrap();
            ignore_store.ignore_for_extension("dict", "py").unwrap();
        }

        #[test]
        fn test_ignored_for_project() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            let project_id1 = 42;
            let project_id2 = 43;
            ignore_store.ignore_for_project("foo", project_id1).unwrap();

            assert!(ignore_store
                .is_ignored_for_project("foo", project_id1)
                .unwrap());
            assert!(!ignore_store
                .is_ignored_for_project("foo", project_id2)
                .unwrap());
        }

        #[test]
        fn test_ignored_for_path() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));
            let foo_rs = RelativePath::from_path_unchecked(PathBuf::from("foo.rs"));
            let id1 = 42;
            let id2 = 43;

            ignore_store.ignore_for_path("foo", id1, &foo_py).unwrap();

            assert!(ignore_store
                .is_ignored_for_path("foo", id1, &foo_py)
                .unwrap());
            // Same project, different file
            assert!(!ignore_store
                .is_ignored_for_path("foo", id1, &foo_rs)
                .unwrap());
            // Same file, different project
            assert!(!ignore_store
                .is_ignored_for_path("foo", id2, &foo_py)
                .unwrap());
        }

        #[test]
        fn test_remove_ignored_happy() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            ignore_store.ignore("foo").unwrap();

            ignore_store.remove_ignored("foo").unwrap();

            assert!(!ignore_store.is_ignored("foo").unwrap());
        }

        #[test]
        fn test_remove_ignored_when_not_ignored() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            assert!(!ignore_store.is_ignored("foo").unwrap());

            assert!(ignore_store.remove_ignored("foo").is_err());
        }

        #[test]
        fn test_remove_ignored_for_extension_happy() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            ignore_store.ignore_for_extension("foo", "py").unwrap();

            ignore_store
                .remove_ignored_for_extension("foo", "py")
                .unwrap();

            assert!(!ignore_store.is_ignored_for_extension("foo", "py").unwrap());
        }

        #[test]
        fn test_remove_ignored_for_extension_when_not_ignored() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            assert!(!ignore_store.is_ignored_for_extension("foo", "py").unwrap());

            assert!(ignore_store
                .remove_ignored_for_extension("foo", "py")
                .is_err());
        }

        #[test]
        fn test_remove_ignored_for_path_happy() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            let project_id = 42;
            let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));

            ignore_store
                .ignore_for_path("foo", project_id, &foo_py)
                .unwrap();

            ignore_store
                .remove_ignored_for_path("foo", project_id, &foo_py)
                .unwrap();

            assert!(!ignore_store
                .is_ignored_for_path("foo", project_id, &foo_py)
                .unwrap());
        }

        #[test]
        fn test_remove_ignored_for_path_when_not_ignored() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            let project_id = 42;
            let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));

            assert!(!ignore_store
                .is_ignored_for_path("foo", project_id, &foo_py)
                .unwrap());

            assert!(ignore_store
                .remove_ignored_for_path("foo", project_id, &foo_py)
                .is_err());
        }

        #[test]
        fn test_remove_ignored_for_project() {
            let mut ignore_store = <$repo>::new_for_tests().unwrap();
            let project_id = 42;
            ignore_store.ignore_for_project("foo", project_id).unwrap();

            ignore_store
                .remove_ignored_for_project("foo", project_id)
                .unwrap();

            assert!(!ignore_store
                .is_ignored_for_project("foo", project_id)
                .unwrap());
        }
    };
}
