use super::*;

fn extract_word_default(word: &str) -> Option<(&str, usize)> {
    let tokenizer = Tokenizer::new(word, ExtractMode::Default);
    tokenizer.extract_word(word)
}

#[test]
fn test_abbrev_re() {
    let re = Regex::new(r"(\p{Lu}+)\p{Lu}\p{Ll}").unwrap();
    assert_eq!(
        re.captures("HTTPError").unwrap().get(1).unwrap().as_str(),
        "HTTP"
    );
}

#[test]
fn test_skip_urls() {
    assert!(extract_word_default("https://foo.com").is_none());
}

#[test]
fn test_skip_emails() {
    assert!(extract_word_default("foo@acme.corp").is_none());
}

#[test]
fn test_skip_mentions() {
    assert!(extract_word_default("@d_merej").is_none());
}

#[test]
fn test_skip_uuid() {
    assert!(extract_word_default("ee54764c-a400-4f56-b335-fe16daaeb114").is_none());
}

#[test]
fn test_skip_sha1s() {
    assert!(extract_word_default("154b879").is_none());
}

#[test]
fn test_remove_numbers() {
    assert_eq!(extract_word_default("foo32").unwrap(), ("foo", 0));
}

#[test]
fn test_remove_numbers_2() {
    assert_eq!(extract_word_default("22xy23").unwrap(), ("xy", 2));
}

#[test]
fn test_snake_case() {
    assert_eq!(extract_word_default("foo_bar").unwrap(), ("foo", 0));
}

#[test]
fn test_snake_case_2() {
    assert_eq!(extract_word_default("__foo").unwrap(), ("foo", 2));
}

#[test]
fn test_snake_case_3() {
    assert_eq!(extract_word_default("foo_").unwrap(), ("foo", 0));
}

#[test]
fn test_ada_case() {
    assert_eq!(extract_word_default("Print_Newline").unwrap(), ("Print", 0));
}

#[test]
fn test_camel_case() {
    assert_eq!(extract_word_default("fooBar").unwrap(), ("foo", 0));
}

#[test]
fn test_pascal_case() {
    assert_eq!(extract_word_default("FooBar").unwrap(), ("Foo", 0));
}

fn get_tokens_default(text: &str) -> Vec<&str> {
    let tokenizer = Tokenizer::new(text, ExtractMode::Default);
    tokenizer.map(|(x, _index)| x).collect()
}

#[test]
fn test_backslash_1() {
    let text = r"one\ntwo";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["one", "two"]);
}

#[test]
fn test_backslash_2() {
    let text = r"\tone\ntwo";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["one", "two"]);
}

#[test]
fn test_backslash_3() {
    let text = r"hello\n\n\nworld";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["hello", "world"]);
}

#[test]
fn test_c_escapes() {
    let text = r"one\ntwo\rthree\tfour";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["one", "two", "three", "four"]);
}

#[test]
fn test_split_identifiers() {
    let text = "hello world foo-bar x y https://toto.com  spam42 'dry-run', foo@acme.corp";
    let actual = get_tokens_default(text);
    assert_eq!(
        &actual,
        &["hello", "world", "foo", "bar", "x", "y", "spam", "dry", "run"]
    );
}

#[test]
fn test_skip_youtube_url() {
    let text = "let url = https://www.youtube.com/watch?v=9LfmrkyP81M; let x = 42";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["let", "url", "let", "x"],);
}

#[test]
fn test_split_camel() {
    let text = "fooBarBaz";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["foo", "Bar", "Baz"]);
}

#[test]
fn test_split_screaming() {
    let text = "SCREAMING_CONSTANT";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["SCREAMING", "CONSTANT"]);
}

#[test]
fn test_split_abbrev() {
    let text = "HttpError";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["Http", "Error"]);
}

#[test]
fn test_split_abbrev_2() {
    let text = "HTTPError";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["HTTP", "Error"]);
}

#[test]
fn test_split_abbrev_3() {
    let text = "URLs";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["URL"]);
}

#[test]
fn test_split_abbrev_4() {
    let text = "ArchivedHTMLTweet";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["Archived", "HTML", "Tweet"]);
}

#[test]
fn test_single_upper_case_letter() {
    let text = "I am";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["I", "am"]);
}

#[test]
fn test_apostrophes() {
    let text = "doesn't matter if it's true";
    let actual = get_tokens_default(text);
    assert_eq!(&actual, &["doesn't", "matter", "if", "it's", "true"]);
}

#[test]
fn test_use_sqlite() {
    let text = "use diesel::sqlite::SqliteConnection;";
    let actual = get_tokens_default(text);
    assert_eq!(
        &actual,
        &["use", "diesel", "sqlite", "Sqlite", "Connection"]
    );
}

fn get_tokens_latex(text: &str) -> Vec<&str> {
    let tokenizer = Tokenizer::new(text, ExtractMode::Latex);
    tokenizer.map(|(x, _index)| x).collect()
}

#[test]
fn test_latex_escape() {
    let text = r"\newpage \tiny";
    let actual = get_tokens_latex(text);
    assert_eq!(&actual, &["newpage", "tiny"]);
}

#[test]
fn test_extract_mode_for_tex_extension() {
    let p = Path::new("foo.tex");
    assert_eq!(ExtractMode::from_path_ext(p), ExtractMode::Latex);
}

fn get_tokens_python(text: &str) -> Vec<&str> {
    let tokenizer = Tokenizer::new(text, ExtractMode::Python);
    tokenizer.map(|(x, _index)| x).collect()
}

#[test]
fn test_python_string_prefix() {
    let text = "message = f'hello, {name}'";
    let actual = get_tokens_python(text);
    assert_eq!(&actual, &["message", "hello", "name"]);
}
