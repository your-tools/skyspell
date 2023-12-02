use super::*;
use std::path::PathBuf;

use textwrap::dedent;

#[test]
fn test_empty_config_is_valid() {
    Config::parse("").unwrap();
}

#[test]
fn test_detailed_error_when_parsing_invalid_kdl_syntax() {
    let input = r#" 
    global {
      one
      two
    }

    extensions {
      "rs {
        fn
      }
    }
    "#;

    let err = Config::parse(input).unwrap_err();
    let message = err.to_string();

    assert!(
        message.contains("valid node name"),
        "bad message: '{message}'"
    );
    assert!(
        message.contains("line 12, column 5"),
        "bad message: '{message}'"
    );
}

// Note: this checks that automatic formatting of the
// skyspell.kdl file is not too bad
fn check<F>(action: F, input: &str, expected: &str)
where
    F: Fn(&mut Config) -> anyhow::Result<()>,
{
    let input = dedent(input);
    let mut ignore_config = Config::parse(&input).unwrap();
    let expected = dedent(expected);
    let expected = expected.trim();
    action(&mut ignore_config).unwrap();
    let actual = ignore_config.to_string();
    let actual = actual.trim();
    assert_eq!(actual, expected, "{actual}");
}

#[test]
fn test_add_global_ignore_to_empty_config() {
    let input = "";

    let action = |x: &mut Config| x.ignore("hello");

    let expected = r#"
            global {
              hello
            }
            "#;

    check(action, input, expected);
}

#[test]
fn test_create_subsection_from_scratch() {
    let input = "global {\n  hello\n}\n";

    let action = |x: &mut Config| x.ignore_for_extension("fn", "rs");

    let expected = r#"
            global {
              hello
            }
            extensions {

              rs {
                fn
              }
            }
            "#;

    check(action, input, expected);
}

#[test]
fn test_add_global_ignore_to_existing_config() {
    let input = r#"
            global {
              abc
              ghi
            }

            project {

            }

            extensions {

            }

            paths {

            }
            "#;

    let action = |x: &mut Config| x.ignore("def");

    let expected = r#"
            global {
              abc
              def
              ghi
            }

            project {

            }

            extensions {

            }

            paths {

            }
            "#;

    check(action, input, expected);
}

#[test]
fn test_remove_word_from_global() {
    let input = r#"
            global {
              abc
              def
            }

            project {

            }

            extensions {

            }

            paths {

            }
            "#;

    let action = |x: &mut Config| x.remove_ignored("def");

    let expected = r#"
            global {
              abc
            }

            project {

            }

            extensions {

            }

            paths {

            }
            "#;

    check(action, input, expected);
}

#[test]
fn test_add_project_ignore() {
    let input = r#"
            global {
              abc
              ghi
            }

            project {

            }

            extensions {

            }

            paths {

            }
            "#;

    let action = |x: &mut Config| x.ignore_for_project("hello");

    let expected = r#"
            global {
              abc
              ghi
            }

            project {
              hello
            }

            extensions {

            }

            paths {

            }
            "#;

    check(action, input, expected);
}

#[test]
fn test_add_ignore_for_new_extension() {
    let input = r#"
            global {
              abc
              ghi
            }

            project {

            }

            extensions {

            }

            paths {

            }
            "#;

    let action = |x: &mut Config| x.ignore_for_extension("fn", "rs");

    let expected = r#"
            global {
              abc
              ghi
            }

            project {

            }

            extensions {
              rs {
                fn
              }


            }

            paths {

            }
            "#;

    check(action, input, expected);
}

#[test]
fn test_add_ignore_for_existing_extension() {
    let input = r#"
            global {
              abc
              ghi
            }

            project {

            }

            extensions {
              rs {
                fn
                impl
              }
        
              tex {
                vfill
              }

            }

            paths {

            }
            "#;

    let action = |x: &mut Config| x.ignore_for_extension("hfill", "tex");

    let expected = r#"
            global {
              abc
              ghi
            }

            project {

            }

            extensions {
              rs {
                fn
                impl
              }
        
              tex {
                hfill
                vfill
              }

            }

            paths {

            }
            "#;

    check(action, input, expected);
}

#[test]
fn test_insert_ignore() {
    let mut config = Config::new_for_tests().unwrap();
    config.ignore("foo").unwrap();

    assert!(config.is_ignored("foo").unwrap());
    assert!(!config.is_ignored("bar").unwrap());
}

#[test]
fn test_lookup_extension() {
    let mut config = Config::new_for_tests().unwrap();
    config.ignore_for_extension("dict", "py").unwrap();

    assert!(config.is_ignored_for_extension("dict", "py").unwrap());
}

#[test]
fn test_insert_ignore_ignore_duplicates() {
    let mut config = Config::new_for_tests().unwrap();
    config.ignore("foo").unwrap();
    config.ignore("foo").unwrap();
}

#[test]
fn test_ignored_for_extension_duplicates() {
    let mut config = Config::new_for_tests().unwrap();
    config.ignore_for_extension("dict", "py").unwrap();
    config.ignore_for_extension("dict", "py").unwrap();
}

#[test]
fn test_ignored_for_project() {
    let mut config = Config::new_for_tests().unwrap();
    config.ignore_for_project("foo").unwrap();

    assert!(config.is_ignored_for_project("foo").unwrap())
}

#[test]
fn test_ignored_for_path() {
    let mut config = Config::new_for_tests().unwrap();
    let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));
    let foo_rs = RelativePath::from_path_unchecked(PathBuf::from("foo.rs"));

    config.ignore_for_path("foo", &foo_py).unwrap();

    assert!(config.is_ignored_for_path("foo", &foo_py).unwrap());
    assert!(!config.is_ignored_for_path("foo", &foo_rs).unwrap());
}

#[test]
fn test_remove_ignored_happy() {
    let mut config = Config::new_for_tests().unwrap();
    config.ignore("foo").unwrap();

    config.remove_ignored("foo").unwrap();

    assert!(!config.is_ignored("foo").unwrap());
}

#[test]
fn test_remove_ignored_when_not_ignored() {
    let mut config = Config::new_for_tests().unwrap();
    assert!(!config.is_ignored("foo").unwrap());

    assert!(config.remove_ignored("foo").is_err());
}

#[test]
fn test_remove_ignored_for_extension_happy() {
    let mut config = Config::new_for_tests().unwrap();
    config.ignore_for_extension("foo", "py").unwrap();

    config
        .remove_ignored_for_extension("foo", "py")
        .unwrap();

    assert!(!config.is_ignored_for_extension("foo", "py").unwrap());
}

#[test]
fn test_remove_ignored_for_extension_when_not_ignored() {
    let mut config = Config::new_for_tests().unwrap();
    assert!(!config.is_ignored_for_extension("foo", "py").unwrap());

    assert!(config
        .remove_ignored_for_extension("foo", "py")
        .is_err());
}

#[test]
fn test_remove_ignored_for_path_happy() {
    let mut config = Config::new_for_tests().unwrap();
    let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));

    config.ignore_for_path("foo", &foo_py).unwrap();

    config
        .remove_ignored_for_path("foo", &foo_py)
        .unwrap();

    assert!(!config.is_ignored_for_path("foo", &foo_py).unwrap());
}

#[test]
fn test_remove_ignored_for_path_when_not_ignored() {
    let mut config = Config::new_for_tests().unwrap();
    let foo_py = RelativePath::from_path_unchecked(PathBuf::from("foo.py"));

    assert!(!config.is_ignored_for_path("foo", &foo_py).unwrap());

    assert!(config
        .remove_ignored_for_path("foo", &foo_py)
        .is_err());
}

#[test]
fn test_remove_ignored_for_project() {
    let mut config = Config::new_for_tests().unwrap();
    config.ignore_for_project("foo").unwrap();

    config.remove_ignored_for_project("foo").unwrap();

    assert!(!config.is_ignored_for_project("foo").unwrap());
}
