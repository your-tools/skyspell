use super::*;
use textwrap::dedent;

#[test]
fn test_error_if_global_is_missing() {
    IgnoreConfig::parse("").unwrap_err();
}

fn check<F>(action: F, input: &str, expected: &str)
where
    F: Fn(&mut IgnoreConfig),
{
    let input = dedent(input);
    let mut ignore_config = IgnoreConfig::parse(&input).unwrap();
    let expected = dedent(expected);
    action(&mut ignore_config);
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

    let action = |x: &mut IgnoreConfig| x.add_global("hello");

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

    let action = |x: &mut IgnoreConfig| x.add_global("def");

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

    let action = |x: &mut IgnoreConfig| x.add_project("hello");

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

    let action = |x: &mut IgnoreConfig| x.add_ignore_for_extension("fn", "rs");

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

    let action = |x: &mut IgnoreConfig| x.add_ignore_for_extension("hfill", "tex");

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
