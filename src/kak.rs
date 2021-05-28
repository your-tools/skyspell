use std::io::Write;
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::checker::lookup_token;
use crate::{Checker, Dictionary, Repo};

pub(crate) struct Error {
    pos: (usize, usize),
    token: String,
}

pub struct KakouneChecker<D: Dictionary, R: Repo> {
    dictionary: D,
    repo: R,
    errors: Vec<Error>,
}

impl<D: Dictionary, R: Repo> Checker for KakouneChecker<D, R> {
    fn is_skipped(&self, path: &Path) -> Result<bool> {
        self.repo.is_skipped(path)
    }

    fn handle_token(&mut self, path: &Path, pos: (usize, usize), token: &str) -> Result<()> {
        let found = lookup_token(&self.dictionary, &self.repo, token, path)?;
        if !found {
            self.errors.push(Error {
                pos,
                token: token.to_owned(),
            });
        }
        Ok(())
    }

    fn success(&self) -> bool {
        true
    }
}

impl<D: Dictionary, R: Repo> KakouneChecker<D, R> {
    pub fn new(dictionary: D, repo: R) -> Self {
        Self {
            dictionary,
            repo,
            errors: vec![],
        }
    }

    fn write_code(&self, f: &mut impl Write) -> Result<()> {
        let kak_timestamp =
            std::env::var("kak_timestamp").map_err(|_| anyhow!("kak_timestamp is not defined"))?;

        let kak_timestamp: usize = kak_timestamp
            .parse()
            .map_err(|_| anyhow!("could not parse kak_timestamp has a positive integer"))?;

        write_error_lines(f, &self.errors)?;
        write_hooks(f)?;
        write_ranges(f, kak_timestamp, &self.errors)?;
        write_status(f, &self.errors)?;

        Ok(())
    }

    pub fn emit_kak_code(&self) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        self.write_code(&mut handle)?;
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

fn write_hooks(f: &mut impl Write) -> Result<()> {
    writeln!(
        f,
        r#"map buffer normal '<ret>' ':<space>kak-spell-jump<ret>'
map buffer normal 'a' ':<space>kak-spell-add-from-spelling-buffer en_US<ret>'
execute-keys <esc> ga"#
    )?;
    Ok(())
}

fn write_error_lines(f: &mut impl Write, errors: &[Error]) -> Result<()> {
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
    let Error { pos, token, .. } = error;
    let (line, start) = pos;
    let end = start + token.len();
    write!(f, "{}.{},{}.{} {}", line, start + 1, line, end, token)?;
    Ok(())
}

fn write_ranges(f: &mut impl Write, timestamp: usize, errors: &[Error]) -> Result<()> {
    write!(f, "set-option buffer spell_errors {} ", timestamp)?;
    for error in errors.iter() {
        write_error_range(f, error)?;
        write!(f, " ")?;
    }
    writeln!(f)?;
    Ok(())
}

fn write_error_range(f: &mut impl Write, error: &Error) -> Result<()> {
    let Error { pos, token, .. } = error;
    let (line, start) = pos;
    write!(f, "{}.{}+{}|Error", line, start + 1, token.len())?;
    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_insert_errors() {
        let error = Error {
            pos: (2, 4),
            token: "foo".to_string(),
        };

        let mut buff: Vec<u8> = vec![];
        write_error_lines(&mut buff, &[error]).unwrap();
        let actual = std::str::from_utf8(&buff).unwrap();
        let expected = r#"edit -scratch *spelling*
execute-keys \% <ret> d i %{2.5,2.7 foo<ret>} <esc> gg
"#;
        assert_eq!(actual, expected);
    }
}
