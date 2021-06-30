use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use regex::{Regex, RegexBuilder};

const GIT_SCISSORS: &str = "# ------------------------ >8 ------------------------";

lazy_static! {
    // We want to match unicode letters and everything that may be contain inside
    // something we want to skip (like an URL)
    static ref TOKEN_RE: Regex = RegexBuilder::new(
        r"
            (
                \w |                          # letters .. or:
                [ \\ + . / : = ? @ _ ~ ' - ]  # all possible chars found in an URL, an email,
                                              # or a word with an apostrophe (like doesn't)
            )+
        "
    ).ignore_whitespace(true).build().expect("syntax error in static regex");

    // We want to match HTTP in HTTPError
    static ref ABBREV_RE: Regex = RegexBuilder::new(
        r"
            (\p{Lu}+)   # Some upper case letters
            \p{Lu}      # An uppercase letter
            \p{Ll}      # A lower case letter
         "
    )
    .ignore_whitespace(true).build().expect("syntax error in static regex");

    // We want to match URL and URLs
    static ref CONSTANT_RE: Regex = RegexBuilder::new(
        r"
        # Only uppercase letters, except maybe a 's' at the end
        ^(\p{Lu}+) s ?$
        "
    )
    .ignore_whitespace(true).build().expect("syntax error in static regex");

    // We want to match 8a1007e (for git sha1)
    static ref HEXA_RE: Regex = RegexBuilder::new(
        r"
        # Only letter a to f and numbers, at list 5 in size
        [a-f0-9]{5,}
        "
    ).ignore_whitespace(true).build().expect("syntax error in static regex");

    // One we've skipped tokens, we want to match any word
    // inside
    static ref IDENT_RE: Regex = RegexBuilder::new(
        r"
        # A word is just a bunch of unicode characters matching
        # the Alphabetic group, possibly inside space escapes like
        # \n or \t, and possibly containing exactly one apostrophe
        (\\[nt])*
        (
            \p{Alphabetic}+ ' \p{Alphabetic}+ | (\p{Alphabetic}+)
        )
        (\\[nt])*
        "
    ).ignore_whitespace(true).build().expect("syntax error in static regex");
}

pub(crate) struct TokenProcessor {
    path: PathBuf,
}

impl TokenProcessor {
    pub(crate) fn new(path: &Path) -> Result<Self> {
        Ok(Self {
            path: path.to_path_buf(),
        })
    }

    pub(crate) fn each_token<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(&str, usize, usize) -> Result<()>,
    {
        let source = File::open(&self.path)
            .with_context(|| format!("Could not open {} for reading", self.path.display()))?;
        let lines = RelevantLines::new(source, self.path.file_name());
        for (i, line) in lines.enumerate() {
            let line =
                line.with_context(|| format!("Error when reading {}", self.path.display()))?;
            let tokenizer = Tokenizer::new(&line);
            for (word, pos) in tokenizer {
                f(word, i + 1, pos)?
            }
        }
        Ok(())
    }
}

struct RelevantLines {
    lines: Lines<BufReader<File>>,
    is_git_message: bool,
}

impl RelevantLines {
    fn new(source: File, filename: Option<&OsStr>) -> Self {
        let is_git_message = filename == Some(OsStr::new("COMMIT_EDITMSG"));
        let reader = BufReader::new(source);
        let lines = reader.lines();
        Self {
            lines,
            is_git_message,
        }
    }
}

impl Iterator for RelevantLines {
    type Item = Result<String, std::io::Error>;

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        let line_result = self.lines.next()?;
        match line_result {
            e @ Err(_) => Some(e),
            Ok(s) if self.is_git_message && s == GIT_SCISSORS => None,
            x => Some(x),
        }
    }
}

struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = (&'a str, usize);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        // Algorithm:
        //   First we find 'tokens', which may contains '-', '/', or '@'
        //   Then we filter out mails, URLs, sha1s and the like from the tokens to get "identifiers"
        //   Then we extract words out of identifiers
        loop {
            let captures = TOKEN_RE.captures(&self.input[self.pos..])?;
            let token_match = captures.get(0).unwrap();
            let token = token_match.as_str();
            let start = token_match.range().start;
            let next_word = extract_word(token);
            if let Some((w, pos)) = next_word {
                let res = (w, self.pos + start + pos);
                self.pos += start + pos + w.len();
                return Some(res);
            } else {
                self.pos += start + token.len();
            }
        }
    }
}

fn extract_word(token: &str) -> Option<(&str, usize)> {
    // Plural constants
    if token == "s" {
        return None;
    }

    // Skip URLs
    if token.contains("://") {
        return None;
    }

    // Skip emails and @mentions
    if token.contains('@') {
        return None;
    }

    if HEXA_RE.find(token).is_some() {
        return None;
    }

    if let Some(captures) = IDENT_RE.captures(token) {
        let ident_match = captures.get(2).unwrap();
        let pos = ident_match.start();
        let ident = ident_match.as_str();
        return word_from_ident(ident, pos);
    }

    None
}

fn word_from_ident(ident: &str, pos: usize) -> Option<(&str, usize)> {
    let mut iter = ident.char_indices();
    // We know the ident cannot be empty because of IDENT_RE
    let (_, first_char) = iter.next().expect("empty ident");
    if first_char.is_lowercase() {
        // camelCase -> camel
        if let Some(p) = ident.find(char::is_uppercase) {
            return Some((&ident[..p], pos));
        }
    }
    if first_char.is_uppercase() {
        // SCREAMING -> SCREAMING
        if let Some(captures) = CONSTANT_RE.captures(ident) {
            let res = captures.get(1).unwrap().as_str();
            return Some((res, pos));
        }

        // HTTPError -> HTTP
        if let Some(captures) = ABBREV_RE.captures(ident) {
            let res = captures.get(1).unwrap().as_str();
            return Some((res, pos));
        }

        let (second_pos, _) = match iter.next() {
            // Single upper letter: return it
            None => return Some((ident, pos)),
            Some(x) => x,
        };

        // PascalCase -> Pascal
        if let Some(next_upper) = (&ident[second_pos..]).find(char::is_uppercase) {
            let res = &ident[..next_upper + second_pos];
            return Some((res, pos));
        }
    }

    Some((ident, pos))
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(extract_word("https://foo.com").is_none());
    }

    #[test]
    fn test_skip_emails() {
        assert!(extract_word("foo@acme.corp").is_none());
    }

    #[test]
    fn test_skip_mentions() {
        assert!(extract_word("@d_merej").is_none());
    }

    #[test]
    fn test_skip_uuid() {
        assert!(extract_word("ee54764c-a400-4f56-b335-fe16daaeb114").is_none());
    }

    #[test]
    fn test_skip_sha1s() {
        assert!(extract_word("154b879").is_none());
    }

    #[test]
    fn test_remove_numbers() {
        assert_eq!(extract_word("foo32").unwrap(), ("foo", 0));
    }

    #[test]
    fn test_remove_numbers_2() {
        assert_eq!(extract_word("22xy23").unwrap(), ("xy", 2));
    }

    #[test]
    fn test_snake_case() {
        assert_eq!(extract_word("foo_bar").unwrap(), ("foo", 0));
    }

    #[test]
    fn test_snake_case_2() {
        assert_eq!(extract_word("__foo").unwrap(), ("foo", 2));
    }

    #[test]
    fn test_snake_case_3() {
        assert_eq!(extract_word("foo_").unwrap(), ("foo", 0));
    }

    #[test]
    fn test_ada_case() {
        assert_eq!(extract_word("Print_Newline").unwrap(), ("Print", 0));
    }

    #[test]
    fn test_camel_case() {
        assert_eq!(extract_word("fooBar").unwrap(), ("foo", 0));
    }

    #[test]
    fn test_pascal_case() {
        assert_eq!(extract_word("FooBar").unwrap(), ("Foo", 0));
    }

    fn get_tokens(text: &str) -> Vec<&str> {
        let tokenizer = Tokenizer::new(&text);
        tokenizer.map(|(x, _index)| x).collect()
    }

    #[test]
    fn test_backslash_1() {
        let text = r"one\ntwo";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["one", "two"]);
    }

    #[test]
    fn test_backslash_2() {
        let text = r"\tone\ntwo";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["one", "two"]);
    }

    #[test]
    fn test_backslash_3() {
        let text = r"hello\n\n\nworld";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["hello", "world"]);
    }

    #[test]
    fn test_latex() {
        let text = r"\section{First chapter}";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["section", "First", "chapter"]);
    }

    #[test]
    fn test_split_identifiers() {
        let text = "hello world foo-bar x y https://toto.com  spam42 'dry-run', foo@acme.corp";
        let actual = get_tokens(text);
        assert_eq!(
            &actual,
            &["hello", "world", "foo", "bar", "x", "y", "spam", "dry", "run"]
        );
    }

    #[test]
    fn test_skip_youtube_url() {
        let text = "let url = https://www.youtube.com/watch?v=9LfmrkyP81M; let x = 42";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["let", "url", "let", "x"],);
    }

    #[test]
    fn test_split_camel() {
        let text = "fooBarBaz";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["foo", "Bar", "Baz"]);
    }

    #[test]
    fn test_split_screaming() {
        let text = "SCREAMING_CONSTANT";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["SCREAMING", "CONSTANT"]);
    }

    #[test]
    fn test_split_abbrev() {
        let text = "HttpError";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["Http", "Error"]);
    }

    #[test]
    fn test_split_abbrev_2() {
        let text = "HTTPError";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["HTTP", "Error"]);
    }

    #[test]
    fn test_split_abbrev_3() {
        let text = "URLs";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["URL"]);
    }

    #[test]
    fn test_single_upper_case_letter() {
        let text = "I am";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["I", "am"]);
    }

    #[test]
    fn test_apostrophes() {
        let text = "doesn't matter if it's true";
        let actual = get_tokens(text);
        assert_eq!(&actual, &["doesn't", "matter", "if", "it's", "true"]);
    }

    #[test]
    fn test_use_sqlite() {
        let text = "use diesel::sqlite::SqliteConnection;";
        let actual = get_tokens(text);
        assert_eq!(
            &actual,
            &["use", "diesel", "sqlite", "Sqlite", "Connection"]
        );
    }
}
