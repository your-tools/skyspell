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
            ^
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
    static ref IDENT_RE_DEFAULT: Regex = RegexBuilder::new(
        r"
        # A word is just a bunch of unicode characters matching
        # the Alphabetic group, possibly inside space escapes like
        # \n or \t, and possibly containing exactly one apostrophe
        (\\[nrt])*
        (
            \p{Alphabetic}+ ' \p{Alphabetic}+ | (\p{Alphabetic}+)
        )
        (\\[nrt])*
        "
    ).ignore_whitespace(true).build().expect("syntax error in static regex");


    static ref IDENT_RE_LATEX: Regex = RegexBuilder::new(
        r"
        # Same as IDENT_RE, without handling \n, \r or \t
        \p{Alphabetic}+ ' \p{Alphabetic}+ | (\p{Alphabetic}+)
        "
    ).ignore_whitespace(true).build().expect("syntax error in static regex");
}

#[rustfmt::skip]
const PYTHON_STRING_PREFIXES: [&str; 24] = [
    "r'", "u'", "R'", "U'", "f'", "F'",
    "fr'", "Fr'", "fR'", "FR'", "rf'", "rF'", "Rf'", "RF'",
    "b'", "B'", "br'", "Br'", "bR'", "BR'", "rb'", "rB'", "Rb'", "RB'",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtractMode {
    Default,
    Latex,
    Python,
}

impl ExtractMode {
    fn from_path_ext(p: &Path) -> Self {
        let ext = match p.extension() {
            None => return ExtractMode::Default,
            Some(e) => e,
        };
        match ext.to_string_lossy().as_ref() {
            "tex" => ExtractMode::Latex,
            "py" => ExtractMode::Python,
            _ => ExtractMode::Default,
        }
    }
}

pub struct TokenProcessor {
    path: PathBuf,
    extract_mode: ExtractMode,
}

impl TokenProcessor {
    pub fn new(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            extract_mode: ExtractMode::from_path_ext(path),
        }
    }

    pub fn each_token<F>(&self, mut f: F) -> Result<()>
    where
        F: FnMut(&str, usize, usize) -> Result<()>,
    {
        let source = File::open(&self.path)
            .with_context(|| format!("Could not open '{}' for reading", self.path.display()))?;
        let lines = RelevantLines::new(source, self.path.file_name());
        for (i, line) in lines.enumerate() {
            let line = line
                .map_err(|e| anyhow!("When reading line from '{}': {}", self.path.display(), e))?;
            let tokenizer = Tokenizer::new(&line, self.extract_mode);
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
    extract_mode: ExtractMode,
}

impl<'a> Tokenizer<'a> {
    fn new(input: &'a str, extract_mode: ExtractMode) -> Self {
        Self {
            input,
            pos: 0,
            extract_mode,
        }
    }

    fn extract_word(&self, token: &'a str) -> Option<(&'a str, usize)> {
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

        let (captures, index) = match self.extract_mode {
            ExtractMode::Latex => (IDENT_RE_LATEX.captures(token), 0),
            ExtractMode::Default | ExtractMode::Python => (IDENT_RE_DEFAULT.captures(token), 2),
        };

        let captures = match captures {
            None => return None,
            Some(c) => c,
        };

        // The `index` comes for the call to `captures()` already, so this
        // should not panic:
        let ident_match = captures.get(index).expect("index should match captures");
        let ident = ident_match.as_str();
        let pos = ident_match.start();
        if self.extract_mode == ExtractMode::Python {
            // We want to skip string prefixes, like in  r'foo'
            let prefix = self.get_python_string_prefix(token);
            if let Some(p) = prefix {
                let ident = ident.get(p.len()..);
                if let Some(i) = ident {
                    return self.word_from_ident(i, p.len());
                }
            }
        }

        self.word_from_ident(ident, pos)
    }

    fn word_from_ident(&self, ident: &'a str, pos: usize) -> Option<(&'a str, usize)> {
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
                // CONSTANT_RE contains at least one group, so this should
                // not panic:
                let res = captures.get(1).unwrap().as_str();
                return Some((res, pos));
            }

            // HTTPError -> HTTP
            if let Some(captures) = ABBREV_RE.captures(ident) {
                // ABBREV_RE contains at least one group, so this should
                // not panic:
                let res = captures.get(1).unwrap().as_str();
                return Some((res, pos));
            }

            let (second_pos, _) = match iter.next() {
                // Single upper letter: return it
                None => return Some((ident, pos)),
                Some(x) => x,
            };

            // PascalCase -> Pascal
            if let Some(next_upper) = ident[second_pos..].find(char::is_uppercase) {
                let res = &ident[..next_upper + second_pos];
                return Some((res, pos));
            }
        }

        Some((ident, pos))
    }

    fn get_python_string_prefix(&self, token: &str) -> Option<&str> {
        PYTHON_STRING_PREFIXES
            .into_iter()
            .find(|&prefix| token.starts_with(prefix))
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
            // .get(0) should never panic
            let token_match = captures.get(0).unwrap();
            let token = token_match.as_str();
            let start = token_match.range().start;
            let next_word = self.extract_word(token);
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

#[cfg(test)]
mod tests;
