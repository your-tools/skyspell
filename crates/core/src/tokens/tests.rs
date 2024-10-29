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
fn test_python_string_prefix_1() {
    let text = "message = f'hello, {name}'";
    let actual = get_tokens_python(text);
    assert_eq!(&actual, &["message", "hello", "name"]);
}

#[test]
fn test_python_string_prefix_2() {
    let text = "r'/path'";
    let actual = get_tokens_python(text);
    // TODO: this should be just ["path"]
    assert_eq!(&actual, &["r", "path"]);
}

mod target_api {
    use std::io::{BufRead, Lines};

    use super::ExtractMode;

    fn get_tokens(line: &str, _extract_mode: ExtractMode) -> Vec<String> {
        let words = line
            .split_ascii_whitespace()
            .map(|x| x.to_owned())
            .collect();
        words
    }

    struct Tokens<B> {
        lines: Lines<B>,
        extract_mode: ExtractMode,
    }

    impl<B> Tokens<B> {
        fn new(lines: Lines<B>, extract_mode: ExtractMode) -> Self {
            Self {
                lines,
                extract_mode,
            }
        }
    }

    impl<B: BufRead> IntoIterator for Tokens<B> {
        type Item = Result<String, std::io::Error>;

        type IntoIter = TokensIterator<B>;

        fn into_iter(self) -> Self::IntoIter {
            TokensIterator::new(self.lines, self.extract_mode)
        }
    }

    struct TokensIterator<B> {
        lines: Lines<B>,
        extract_mode: ExtractMode,
        #[allow(dead_code)]
        line_number: usize,
        current_line: String,
        tokens: Vec<String>,
        token_index: usize,
        new_line: bool,
    }

    impl<B> TokensIterator<B> {
        fn new(lines: Lines<B>, extract_mode: ExtractMode) -> Self {
            Self {
                lines,
                extract_mode,
                line_number: 0,
                tokens: vec![],
                token_index: 0,
                current_line: String::new(),
                new_line: true,
            }
        }
    }

    impl<B: BufRead> Iterator for TokensIterator<B> {
        type Item = Result<String, std::io::Error>;

        fn next(&mut self) -> Option<Self::Item> {
            loop {
                if self.new_line {
                    let next_line = self.lines.next();
                    let line_result = match next_line {
                        None => return None,
                        Some(l) => l,
                    };
                    let line = match line_result {
                        Ok(l) => l,
                        Err(e) => return Some(Err(e)),
                    };
                    self.current_line = line;
                    self.tokens = get_tokens(&self.current_line, self.extract_mode);
                }

                let token = self.tokens.get(self.token_index);
                self.token_index += 1;
                match token {
                    Some(t) => {
                        self.new_line = false;
                        return Some(Ok(t.to_owned()));
                    }
                    None => {
                        self.new_line = true;
                        self.token_index = 0;
                    }
                }
            }
        }
    }

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
}
