use anyhow::Result;
use itertools::Itertools;
use std::path::PathBuf;

use skyspell_core::repository::RepositoryHandler;
use skyspell_core::Checker;
use skyspell_core::OperatingSystemIO;
use skyspell_core::{Dictionary, Repository};
use skyspell_core::{Project, ProjectPath, RelativePath};

use crate::io::KakouneIO;

pub(crate) struct Error {
    pos: (usize, usize),
    buffer: String,
    full_path: PathBuf,
    token: String,
}

pub(crate) struct KakouneChecker<D: Dictionary, R: Repository, S: OperatingSystemIO> {
    // Note: pub(crate) because KakCli needs read and write access to those fields
    pub(crate) kakoune_io: KakouneIO<S>,
    pub(crate) repository_handler: RepositoryHandler<R>,

    project: Project,
    dictionary: D,
    errors: Vec<Error>,
}

impl<D: Dictionary, R: Repository, S: OperatingSystemIO> Checker for KakouneChecker<D, R, S> {
    // bufname, line, column
    type Context = (String, usize, usize);

    fn handle_error(
        &mut self,
        error: &str,
        path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        let (buffer, line, column) = context;
        let pos = (*line, *column);
        let full_path = self.project.path().as_ref().join(&path);
        self.errors.push(Error {
            full_path,
            pos,
            buffer: buffer.to_string(),
            token: error.to_string(),
        });
        Ok(())
    }

    fn success(&self) -> Result<()> {
        // This checker is always successful, unless we can't fill up the *spelling*
        // buffer for some reason (but this is caught earlier)
        Ok(())
    }

    fn repository(&self) -> &dyn Repository {
        &self.repository_handler.repository
    }

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn project(&self) -> &Project {
        &self.project
    }
}

impl<D: Dictionary, R: Repository, S: OperatingSystemIO> KakouneChecker<D, R, S> {
    pub(crate) fn new(
        project_path: ProjectPath,
        dictionary: D,
        mut repository: R,
        kakoune_io: KakouneIO<S>,
    ) -> Result<Self> {
        let project = repository.ensure_project(&project_path)?;
        let repository_handler = RepositoryHandler::new(repository);
        Ok(Self {
            project,
            dictionary,
            kakoune_io,
            repository_handler,
            errors: vec![],
        })
    }

    fn print(&self, command: &str) {
        self.kakoune_io.print(command)
    }

    pub(crate) fn write_code(&self) -> Result<()> {
        let kak_timestamp = self.kakoune_io.get_timestamp()?;
        self.write_spelling_buffer();
        self.kakoune_io.goto_previous_buffer();
        self.write_ranges(kak_timestamp);
        self.write_status();

        Ok(())
    }

    fn write_status(&self) {
        let project_path = &self.project.path();
        let errors_count = self.errors.len();
        self.print(&format!(
            "set global skyspell_error_count {}\n",
            errors_count
        ));
        match errors_count {
            0 => self.print(&format!(
                "echo -markup {}: {{green}}No spelling errors\n",
                project_path
            )),
            1 => self.print(&format!(
                "echo -markup {}: {{red}}1 spelling error\n",
                project_path
            )),
            n => self.print(&format!(
                "echo -markup {}: {{red}}{} spelling errors\n",
                project_path, n,
            )),
        }
    }

    fn write_spelling_buffer(&self) {
        // Open buffer
        self.print("edit -scratch *spelling*\n");

        // Delete everything
        self.print(r"execute-keys \% <ret> d ");

        // Insert all errors
        self.print("i %{");

        for error in self.errors.iter() {
            self.write_error(error);
            self.print("<ret>");
        }
        self.print("} ");

        // Back to top
        self.print("<esc> gg\n");
    }

    fn write_error(&self, error: &Error) {
        let Error {
            pos,
            token,
            full_path,
            ..
        } = error;
        let (line, start) = pos;
        let end = start + token.len();
        self.print(&format!(
            "{}: {}.{},{}.{} {}",
            full_path.display(),
            line,
            // Columns start at 1
            start + 1,
            line,
            end,
            token
        ));
    }

    fn write_ranges(&self, timestamp: usize) {
        for (buffer, group) in &self.errors.iter().group_by(|e| &e.buffer) {
            if self
                .kakoune_io
                .get_boolean_option("skyspell_underline_errors")
                .expect("skyspell_underline_errors should always be set")
            {
                self.print(&format!("set-face buffer={} Error ,,red+c\n", buffer));
            }
            self.print(&format!(
                "set-option buffer={} spell_errors {} ",
                buffer, timestamp
            ));
            for error in group {
                self.write_error_range(error);
                self.print(" ");
            }
            self.print("\n");
        }
    }

    fn write_error_range(&self, error: &Error) {
        let Error { pos, token, .. } = error;
        let (line, start) = pos;
        self.print(&format!("{}.{}+{}|Error", line, start + 1, token.len()));
    }
}
