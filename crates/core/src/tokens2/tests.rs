use super::*;

#[test]
fn test_target_api() {
    let text = r#" one two
            three
            four
        "#;
    let vec = text.as_bytes();
    let lines: std::io::Lines<_> = vec.lines();
    let tokens = Tokens::new(lines, ExtractMode::Default);
    let tokens: Result<Vec<_>, _> = tokens.into_iter().collect();
    let tokens = tokens.unwrap();
    assert_eq!(
        tokens,
        [
            "one".to_owned(),
            "two".to_owned(),
            "three".to_owned(),
            "four".to_owned()
        ]
    );
}
