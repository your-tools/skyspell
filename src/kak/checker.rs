use std::io::Write;
use std::path::PathBuf;

use anyhow::{anyhow, Result};
use itertools::Itertools;

use crate::kak::helpers::*;
use crate::Checker;
use crate::SQLRepository;
use crate::{Dictionary, Repository};
use crate::{Project, RelativePath};

pub(crate) const SKYSPELL_LANG_OPT: &str = "skyspell_lang";
pub(crate) const SKYSPELL_PROJECT_OPT: &str = "skyspell_project";

pub(crate) fn get_lang() -> Result<String> {
    get_option(SKYSPELL_LANG_OPT)
}

pub(crate) fn open_repository() -> Result<crate::SQLRepository> {
    let lang = get_lang()?;
    SQLRepository::open(&lang)
}

pub(crate) struct Error {
    pos: (usize, usize),
    buffer: String,
    path: PathBuf,
    token: String,
}

pub(crate) struct KakouneChecker<D: Dictionary, R: Repository> {
    project: Project,
    dictionary: D,
    repository: R,
    errors: Vec<Error>,
}

impl<D: Dictionary, R: Repository> Checker for KakouneChecker<D, R> {
    // bufname, line, column
    type Context = (String, usize, usize);

    fn handle_error(
        &mut self,
        error: &str,
        path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        let (buffer, line, column) = context;
        let full_path = std::fs::canonicalize(path)?;
        let pos = (*line, *column);
        self.errors.push(Error {
            path: full_path,
            pos,
            buffer: buffer.to_string(),
            token: error.to_string(),
        });
        Ok(())
    }

    fn success(&self) -> bool {
        true
    }

    fn repository(&self) -> &dyn Repository {
        &self.repository
    }

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn project(&self) -> &Project {
        &self.project
    }
}

impl<D: Dictionary, R: Repository> KakouneChecker<D, R> {
    pub(crate) fn new(project: Project, dictionary: D, mut repository: R) -> Result<Self> {
        repository.ensure_project(&project)?;
        Ok(Self {
            project,
            dictionary,
            repository,
            errors: vec![],
        })
    }

    fn write_code(&self, f: &mut impl Write) -> Result<()> {
        let kak_timestamp =
            std::env::var("kak_timestamp").map_err(|_| anyhow!("kak_timestamp is not defined"))?;

        let kak_timestamp = kak_timestamp
            .parse::<usize>()
            .map_err(|_| anyhow!("could not parse kak_timestamp has a positive integer"))?;

        write_spelling_buffer(f, &self.errors)?;
        goto_previous_buffer();
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
    let project = get_project()?;
    match errors.len() {
        0 => write!(f, "echo -markup {}: {{green}}No spelling errors", project),
        1 => write!(f, "echo -markup {}: {{red}}1 spelling error", project),
        n => write!(f, "echo -markup {}: {{red}}{} Spelling errors", project, n,),
    }?;
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
        pos, token, path, ..
    } = error;
    let (line, start) = pos;
    let end = start + token.len();
    write!(
        f,
        "{}: {}.{},{}.{} {}",
        path.display(),
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
            write!(f, " ")?;
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

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_insert_errors() {
        let error = Error {
            pos: (2, 4),
            buffer: "hello.js".to_string(),
            path: PathBuf::from("/path/to/hello.js"),
            token: "foo".to_string(),
        };

        let mut buff: Vec<u8> = vec![];
        write_spelling_buffer(&mut buff, &[error]).unwrap();
        let actual = std::str::from_utf8(&buff).unwrap();
        let expected = r#"edit -scratch *spelling*
execute-keys \% <ret> d i %{/path/to/hello.js: 2.5,2.7 foo<ret>} <esc> gg
"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_write_ranges() {
        let err1 = Error {
            pos: (2, 4),
            buffer: "foo.js".to_string(),
            path: PathBuf::from("/path/to/foo.js"),
            token: "foo".to_string(),
        };

        let err2 = Error {
            pos: (3, 6),
            buffer: "foo.js".to_string(),
            path: PathBuf::from("/path/to/foo.js"),
            token: "bar".to_string(),
        };

        let err3 = Error {
            pos: (1, 5),
            path: PathBuf::from("/path/to/bar.js"),
            buffer: "bar.js".to_string(),
            token: "spam".to_string(),
        };

        let mut buff: Vec<u8> = vec![];
        write_ranges(&mut buff, 42, &[err1, err2, err3]).unwrap();
        let actual = std::str::from_utf8(&buff).unwrap();
        let expected = "\
set-option buffer=foo.js spell_errors 42 2.5+3|Error 3.7+3|Error \n\
set-option buffer=bar.js spell_errors 42 1.6+4|Error \n";
        assert_eq!(actual, expected);
    }
}
