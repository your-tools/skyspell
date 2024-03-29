use anyhow::Result;
use itertools::Itertools;
use std::path::PathBuf;

use skyspell_core::Checker;
use skyspell_core::Dictionary;
use skyspell_core::OperatingSystemIO;
use skyspell_core::StorageBackend;
use skyspell_core::{Project, RelativePath};

use crate::io::KakouneIO;

pub struct Error {
    pub pos: (usize, usize),
    pub buffer: String,
    pub full_path: PathBuf,
    pub token: String,
}

pub struct KakouneChecker<D: Dictionary, S: OperatingSystemIO> {
    kakoune_io: KakouneIO<S>,
    storage_backend: StorageBackend,
    project: Project,
    dictionary: D,
    errors: Vec<Error>,
}

impl<D: Dictionary, S: OperatingSystemIO> Checker for KakouneChecker<D, S> {
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
        let full_path = self.project.path().as_ref().join(path);
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

    fn storage_backend(&mut self) -> &mut StorageBackend {
        &mut self.storage_backend
    }

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn project(&self) -> &Project {
        &self.project
    }
}

impl<D: Dictionary, S: OperatingSystemIO> KakouneChecker<D, S> {
    pub fn new(
        project: Project,
        dictionary: D,
        storage_backend: StorageBackend,
        kakoune_io: KakouneIO<S>,
    ) -> Result<Self> {
        Ok(Self {
            project,
            dictionary,
            kakoune_io,
            storage_backend,
            errors: vec![],
        })
    }

    pub fn io(&self) -> &KakouneIO<S> {
        &self.kakoune_io
    }

    pub fn print(&self, command: &str) {
        self.kakoune_io.print(command)
    }

    pub fn storage_backend_mut(&mut self) -> &mut StorageBackend {
        &mut self.storage_backend
    }

    pub fn write_code(&self) -> Result<()> {
        let kak_timestamp = self.kakoune_io.get_timestamp()?;
        self.write_spelling_buffer();
        self.write_ranges(kak_timestamp);
        self.write_status();

        Ok(())
    }

    pub fn write_status(&self) {
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
        // Only write in draft mode
        self.print("evaluate-commands -draft %{");

        // Open buffer
        self.print("edit -scratch *spelling*\n");

        // Delete everything
        self.print(r"execute-keys -draft \% <ret> d ");

        // Insert all errors
        self.print("i %{");

        for error in self.errors.iter() {
            self.write_error(error);
            self.print("<ret>");
        }
        self.print("} ");

        // End draft commands, this leaves the cursor where it was,
        // and does not pollute buffer list or undo
        self.print("<esc>}\n");
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
            self.print(&format!(
                "set-option %{{buffer={}}} skyspell_errors {} ",
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
        self.print(&format!(
            "{}.{}+{}|SpellingError",
            line,
            start + 1,
            token.len()
        ));
    }
}

#[cfg(test)]
pub(crate) mod tests;
