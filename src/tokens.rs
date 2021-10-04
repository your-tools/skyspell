use std::ffi::OsStr;
use std::fs::File;
use std::io::{BufRead, BufReader, Lines};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
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
    pub(crate) fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
        }
    }

    pub(crate) fn each_token<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(&str, usize, usize) -> Result<()>,
    {
        let source = File::open(&self.path)
            .with_context(|| format!("Could not open {} for reading", self.path.display()))?;
        let lines = RelevantLines::new(source, self.path.file_name());
        for (i, line) in lines.enumerate() {
            let line = line.map_err(|e| anyhow!("When reading line: {}", e))?;
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
mod tests;
