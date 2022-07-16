use super::*;

use crate::test_ignore_store;

use textwrap::dedent;

#[test]
fn test_error_if_global_is_missing() {
    IgnoreConfig::parse(None, "").unwrap_err();
}

// Note: this checks that automatic formatting of the
// skyspell.kdl file is not too bad
fn check<F>(action: F, input: &str, expected: &str)
where
    F: Fn(&mut IgnoreConfig) -> anyhow::Result<()>,
{
    let input = dedent(input);
    let mut ignore_config = IgnoreConfig::parse(None, &input).unwrap();
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
            
            paths {

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

            paths {

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

            paths {

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

            paths {

            }
            "#;

    check(&action, input, expected);
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

    let action = |x: &mut IgnoreConfig| x.remove_ignored("def");

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

            paths {

            }
            "#;

    let action = |x: &mut IgnoreConfig| x.ignore_for_project("hello", MAGIC_PROJECT_ID);

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

            paths {

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

            paths {

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

            paths {

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

            paths {

            }
            "#;

    check(&action, input, expected);
}

test_ignore_store!(IgnoreConfig);
