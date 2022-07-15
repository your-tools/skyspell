use super::*;
use textwrap::dedent;

#[test]
fn test_error_if_global_is_missing() {
    IgnoreConfig::parse("").unwrap_err();
}

// Note: this checks that automatic formatting of the
// skyspell.kdl file is not too bad
fn check<F>(action: F, input: &str, expected: &str)
where
    F: Fn(&mut IgnoreConfig) -> anyhow::Result<()>,
{
    let input = dedent(input);
    let mut ignore_config = IgnoreConfig::parse(&input).unwrap();
    let expected = dedent(expected);
    action(&mut ignore_config).unwrap();
    let actual = ignore_config.to_string();
    assert_eq!(actual, expected, "{actual}");
}

#[test]
fn test_add_global_ignore_to_empty_config() {
    let input = r#"
            global {
            }

            project {

            }

            extensions {

            }
            "#;

    let action = |x: &mut IgnoreConfig| x.ignore("hello");

    let expected = r#"
            global {
              hello
            }

            project {

            }

            extensions {

            }
            "#;

    check(&action, input, expected);
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
            "#;

    let action = |x: &mut IgnoreConfig| x.ignore("def");

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
            "#;

    check(&action, input, expected);
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
            "#;

    let action = |x: &mut IgnoreConfig| x.ignore_for_project("hello", PROJECT_ID);

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
            "#;

    check(&action, input, expected);
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
            "#;

    let action = |x: &mut IgnoreConfig| x.ignore_for_extension("fn", "rs");

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
            "#;

    check(&action, input, expected);
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
            "#;

    let action = |x: &mut IgnoreConfig| x.ignore_for_extension("hfill", "tex");

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
            "#;

    check(&action, input, expected);
}

#[test]
fn test_is_ignored() {
    let mut ignore_config = IgnoreConfig::new();
    ignore_config.ignore("hello").unwrap();
    let actual = ignore_config.is_ignored("hello").unwrap();
    assert_eq!(actual, true);
}

#[test]
fn test_is_ignored_for_project() {
    let mut ignore_config = IgnoreConfig::new();
    ignore_config
        .ignore_for_project("hello", PROJECT_ID)
        .unwrap();
    let actual = ignore_config
        .is_ignored_for_project("hello", PROJECT_ID)
        .unwrap();
    assert_eq!(actual, true);
}

#[test]
fn test_is_ignored_for_extension() {
    let mut ignore_config = IgnoreConfig::new();
    ignore_config.ignore("hello").unwrap();
    ignore_config.ignore_for_extension("fn", "rs").unwrap();
    let actual = ignore_config.is_ignored_for_extension("fn", "rs").unwrap();
    assert_eq!(actual, true);
}

use crate::test_repository;
test_repository!(IgnoreConfig);
