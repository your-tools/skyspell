use regex::Regex;

lazy_static! {
    static ref ABBREV_RE: Regex = Regex::new(r"(\p{Lu}+)\p{Lu}\p{Ll}").unwrap();
    static ref CONSTANT_RE: Regex = Regex::new(r"^(\p{Lu}+)s?$").unwrap();
    static ref HEXA_RE: Regex = Regex::new(r"[a-f0-9]{5,}").unwrap();
    static ref IDENT_RE: Regex = Regex::new(r"\p{Alphabetic}+").unwrap();
    static ref TOKEN_RE: Regex = Regex::new(r"(\w|-|_|:|/|\.|@)+").unwrap();
}

pub struct Tokenizer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
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

    if let Some(ident_match) = IDENT_RE.find(token) {
        let ident = ident_match.as_str();
        let pos = ident_match.start();
        return word_from_ident(ident, pos);
    }

    None
}

fn word_from_ident(ident: &str, pos: usize) -> Option<(&str, usize)> {
    // We know the ident cannot be empty because of IDENT_RE
    let mut iter = ident.char_indices();
    let (_, first_char) = iter.next().unwrap();
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

    #[test]
    fn test_split_identifiers() {
        let text = "hello world foo-bar x y https://toto.com  spam42 'dry-run', foo@acme.corp";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(
            &actual,
            &["hello", "world", "foo", "bar", "x", "y", "spam", "dry", "run"]
        );
    }

    #[test]
    fn test_split_camel() {
        let text = "fooBarBaz";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(&actual, &["foo", "Bar", "Baz"]);
    }

    #[test]
    fn test_split_screaming() {
        let text = "SCREAMING_CONSTANT";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(&actual, &["SCREAMING", "CONSTANT"]);
    }

    #[test]
    fn test_split_abbrev() {
        let text = "HttpError";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(&actual, &["Http", "Error"]);
    }

    #[test]
    fn test_split_abbrev_2() {
        let text = "HTTPError";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(&actual, &["HTTP", "Error"]);
    }

    #[test]
    fn test_split_abbrev_3() {
        let text = "URLs";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(&actual, &["URL"]);
    }

    #[test]
    fn test_single_upper_case_letter() {
        let text = "I am";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(&actual, &["I", "am"]);
    }

    #[test]
    fn test_use_sqlite() {
        let text = "use diesel::sqlite::SqliteConnection;";
        let tokenizer = Tokenizer::new(&text);
        let actual: Vec<_> = tokenizer.map(|(x, _index)| x).collect();
        assert_eq!(
            &actual,
            &["use", "diesel", "sqlite", "Sqlite", "Connection"]
        );
    }
}
