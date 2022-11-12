use super::*;

use crate::test_ignore_store;

use textwrap::dedent;

#[test]
fn test_empty_config_is_valid() {
    IgnoreConfig::parse(None, "").unwrap();
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

    let err = IgnoreConfig::parse(None, input).unwrap_err();
    let message = err.to_string();

    assert!(
        message.contains("valid node name"),
        "bad message: '{message}'"
    );
    assert!(message.contains("12:5"), "bad message: '{message}'");
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
    let expected = expected.trim();
    action(&mut ignore_config).unwrap();
    let actual = ignore_config.to_string();
    let actual = actual.trim();
    assert_eq!(actual, expected, "{actual}");
}

#[test]
fn test_add_global_ignore_to_empty_config() {
    let input = "";

    let action = |x: &mut IgnoreConfig| x.ignore("hello");

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

    let action = |x: &mut IgnoreConfig| x.ignore_for_extension("fn", "rs");

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

    check(action, input, expected);
}

test_ignore_store!(IgnoreConfig);
