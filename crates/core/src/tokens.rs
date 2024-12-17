use anyhow::{anyhow, Result};
use regex::{Regex, RegexBuilder};
use std::collections::HashSet;
use std::io::BufRead;

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
    fn from_extension(extension: &str) -> Self {
        match extension {
            "tex" => ExtractMode::Latex,
            "py" => ExtractMode::Python,
            _ => ExtractMode::Default,
        }
    }
}

struct Tokenizer<'input, 'skipped> {
    input: &'input str,
    pos: usize,
    extract_mode: ExtractMode,
    skipped: &'skipped HashSet<String>,
}

impl<'input, 'skipped> Tokenizer<'input, 'skipped> {
    fn new(
        input: &'input str,
        extract_mode: ExtractMode,
        skipped: &'skipped HashSet<String>,
    ) -> Self {
        Self {
            input,
            pos: 0,
            extract_mode,
            skipped,
        }
    }

    fn extract_word(&self, token: &'input str) -> Option<(&'input str, usize)> {
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

    fn word_from_ident(&self, ident: &'input str, pos: usize) -> Option<(&'input str, usize)> {
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

impl<'input, 'skipped> Iterator for Tokenizer<'input, 'skipped> {
    type Item = (&'input str, usize);

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
            if self.skipped.contains(token) {
                self.pos += start + token.len();
                continue;
            }
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

#[derive(Debug)]
pub struct Token {
    pub text: String,
    pub pos: (usize, usize),
}

impl Token {
    pub(crate) fn new(text: &str, pos: (usize, usize)) -> Self {
        Self {
            text: text.to_string(),
            pos,
        }
    }

    fn cloned(&self) -> Self {
        Self {
            text: self.text.to_string(),
            pos: self.pos,
        }
    }
}

pub struct TokenProcessor<R: BufRead> {
    reader: R,
    file_name: String,
    current_line: String,
    current_tokens: Vec<Token>,
    extract_mode: ExtractMode,
    word_index: usize,
    line_index: usize,
    skipped_tokens: HashSet<String>,
    is_git_message: bool,
}

impl<R: BufRead> TokenProcessor<R> {
    pub fn new(reader: R, file_name: &str) -> Self {
        let is_git_message = file_name == "COMMIT_EDITMSG";
        let extension = file_name.rsplit(".").next().unwrap_or_default();
        let extract_mode = ExtractMode::from_extension(extension);

        Self {
            reader,
            file_name: file_name.to_owned(),
            extract_mode,
            current_line: String::new(),
            current_tokens: Vec::new(),
            word_index: 0,
            line_index: 0,
            skipped_tokens: HashSet::new(),
            is_git_message,
        }
    }

    pub fn skip_tokens(&mut self, tokens: &[String]) {
        for token in tokens {
            self.skipped_tokens.insert(token.to_string());
        }
    }

    // Return Ok(true) if reached end of file
    fn read_next_line(&mut self) -> Result<bool> {
        self.current_line.clear();
        self.line_index += 1;
        let bytes_read = self.reader.read_line(&mut self.current_line);
        match bytes_read {
            Err(read_error) => Err(anyhow!(
                "Error when reading: '{}': {read_error}",
                self.file_name,
            )),
            Ok(n) => Ok(n == 0),
        }
    }

    // Return Ok(true) if reached end of file
    fn on_last_token(&mut self) -> Result<bool> {
        let is_end_of_file = self.read_next_line()?;
        if is_end_of_file {
            return Ok(true);
        }
        if self.is_git_message && self.current_line.trim() == GIT_SCISSORS {
            return Ok(true);
        }
        self.extract_tokens();
        Ok(false)
    }

    fn extract_tokens(&mut self) {
        self.word_index = 0;
        let tokenizer = Tokenizer::new(&self.current_line, self.extract_mode, &self.skipped_tokens);
        self.current_tokens = tokenizer
            .map(|(token, column)| Token::new(token, (self.line_index, column)))
            .collect();
    }
}

impl<R: BufRead> Iterator for TokenProcessor<R> {
    type Item = Result<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let next_token = self.current_tokens.get(self.word_index);
            match next_token {
                None => {
                    let on_last_token = self.on_last_token();
                    if let Err(e) = on_last_token {
                        return Some(Err(e));
                    }
                    let is_end_of_file = on_last_token.unwrap();
                    if is_end_of_file {
                        return None;
                    }
                }
                Some(token) => {
                    let token = token.cloned();
                    self.word_index += 1;
                    return Some(Ok(token));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
