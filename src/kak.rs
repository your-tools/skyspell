use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Result};
use itertools::Itertools;

use crate::checker::lookup_token;
use crate::{Dictionary, Repo};

pub(crate) struct Error {
    pos: (usize, usize),
    buffer: String,
    token: String,
}

pub struct KakouneChecker<D: Dictionary, R: Repo> {
    dictionary: D,
    repo: R,
    errors: Vec<Error>,
}

impl<D: Dictionary, R: Repo> KakouneChecker<D, R> {
    pub fn new(dictionary: D, repo: R) -> Self {
        Self {
            dictionary,
            repo,
            errors: vec![],
        }
    }

    pub fn is_skipped(&self, path: &Path) -> Result<bool> {
        self.repo.is_skipped(path)
    }

    pub fn handle_token(
        &mut self,
        path: &Path,
        buffer: &str,
        pos: (usize, usize),
        token: &str,
    ) -> Result<()> {
        let found = lookup_token(&self.dictionary, &self.repo, token, path)?;
        if !found {
            self.errors.push(Error {
                pos,
                buffer: buffer.to_owned(),
                token: token.to_owned(),
            });
        }
        Ok(())
    }

    fn write_code(&self, f: &mut impl Write) -> Result<()> {
        let kak_timestamp =
            std::env::var("kak_timestamp").map_err(|_| anyhow!("kak_timestamp is not defined"))?;

        let kak_timestamp = kak_timestamp
            .parse::<usize>()
            .map_err(|_| anyhow!("could not parse kak_timestamp has a positive integer"))?;

        let lang = self.dictionary.lang();

        write_spelling_buffer(f, &self.errors)?;
        write_hooks(f, lang)?;
        write_ranges(f, kak_timestamp, &self.errors)?;
        write_status(f, &self.errors)?;

        Ok(())
    }

    pub fn emit_kak_code(&self) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        self.write_code(&mut handle)?;

        /* debug start */
        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        self.write_code(&mut handle)?;
        /* debug end */
        Ok(())
    }
}

fn write_status(f: &mut impl Write, errors: &[Error]) -> Result<()> {
    match errors.len() {
        0 => write!(f, "echo -markup {{green}} no spelling errors"),
        1 => write!(f, "echo -markup {{red}} 1 spelling error"),
        n => write!(f, "echo -markup {{red}} {} spelling errors", n),
    }?;
    Ok(())
}

// TODO: can we do it without passing the lang back and forth?
fn write_hooks(f: &mut impl Write, lang: &str) -> Result<()> {
    writeln!(
        f,
        r#"map buffer normal '<ret>' ':<space>kak-spell-buffer-action {lang} jump<ret>'
map buffer normal 'a' ':<space>kak-spell-buffer-action {lang} add-global<ret>'
map buffer normal 'e' ':<space>kak-spell-buffer-action {lang} add-extension<ret>'
map buffer normal 'f' ':<space>kak-spell-buffer-action {lang} add-file<ret>'
map buffer normal 'n' ':<space>kak-spell-buffer-action {lang} skip-name<ret>'
map buffer normal 's' ':<space>kak-spell-buffer-action {lang} skip-file<ret>'
execute-keys <esc> ga"#,
        lang = lang
    )?;
    Ok(())
}

fn write_spelling_buffer(f: &mut impl Write, errors: &[Error]) -> Result<()> {
    // Open buffer
    writeln!(f, "edit -scratch *spelling*")?;

    // Delete everything
    write!(f, r"execute-keys \% <ret> d ")?;

    // Insert all errors
    write!(f, "i %{{")?;

    for error in errors.iter() {
        write_error(f, error)?;
        write!(f, "<ret>")?;
    }
    write!(f, "}} ")?;

    // Back to top
    writeln!(f, "<esc> gg")?;
    Ok(())
}

fn write_error(f: &mut impl Write, error: &Error) -> Result<()> {
    let Error {
        pos, token, buffer, ..
    } = error;
    let (line, start) = pos;
    let end = start + token.len();
    write!(
        f,
        "{}: {}.{},{}.{} {}",
        buffer,
        line,
        start + 1,
        line,
        end,
        token
    )?;
    Ok(())
}

fn write_ranges(f: &mut impl Write, timestamp: usize, errors: &[Error]) -> Result<()> {
    for (buffer, group) in &errors.iter().group_by(|e| &e.buffer) {
        write!(
            f,
            "set-option buffer={} spell_errors {} ",
            buffer, timestamp
        )?;
        for error in group {
            write_error_range(f, error)?;
            write!(f, "  ")?;
        }
        writeln!(f)?;
    }
    Ok(())
}

fn write_error_range(f: &mut impl Write, error: &Error) -> Result<()> {
    let Error { pos, token, .. } = error;
    let (line, start) = pos;
    write!(f, "{}.{}+{}|Error", line, start + 1, token.len())?;
    Ok(())
}

pub fn get_previous_selection(
    cursor: (usize, usize),
    ranges: &[(usize, usize, usize)],
) -> Option<&(usize, usize, usize)> {
    let (cursor_line, cursor_col) = cursor;
    for range in ranges.iter().rev() {
        let &(start_line, _start_col, end_col) = range;

        if start_line > cursor_line {
            continue;
        }

        if start_line == cursor_line && end_col >= cursor_col {
            continue;
        }
        return Some(range);
    }

    // If we reach there, return the first error (auto-wrap)
    ranges.iter().last()
}

pub fn get_next_selection(
    cursor: (usize, usize),
    ranges: &[(usize, usize, usize)],
) -> Option<&(usize, usize, usize)> {
    let (cursor_line, cursor_col) = cursor;
    for range in ranges.iter() {
        let &(start_line, _start_col, end_col) = range;

        if start_line < cursor_line {
            continue;
        }

        if start_line == cursor_line && end_col <= cursor_col {
            continue;
        }
        return Some(range);
    }

    ranges.iter().next()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_insert_errors() {
        let error = Error {
            pos: (2, 4),
            buffer: "hello.js".to_string(),
            token: "foo".to_string(),
        };

        let mut buff: Vec<u8> = vec![];
        write_spelling_buffer(&mut buff, &[error]).unwrap();
        let actual = std::str::from_utf8(&buff).unwrap();
        let expected = r#"edit -scratch *spelling*
execute-keys \% <ret> d i %{hello.js: 2.5,2.7 foo<ret>} <esc> gg
"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_write_ranges() {
        let err1 = Error {
            pos: (2, 4),
            buffer: "foo.js".to_string(),
            token: "foo".to_string(),
        };

        let err2 = Error {
            pos: (3, 6),
            buffer: "foo.js".to_string(),
            token: "bar".to_string(),
        };

        let err3 = Error {
            pos: (1, 5),
            buffer: "spam.js".to_string(),
            token: "baz".to_string(),
        };

        let mut buff: Vec<u8> = vec![];
        write_ranges(&mut buff, 42, &[err1, err2, err3]).unwrap();
        let actual = std::str::from_utf8(&buff).unwrap();
        dbg!(actual);
    }

    #[test]
    fn goto_next_no_errors() {
        let pos = (1, 21);
        let ranges = [(1, 12, 19), (2, 19, 27)];
        let actual = get_previous_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(1, 12, 19));
    }
}
