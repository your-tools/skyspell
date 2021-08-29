use anyhow::Result;
use itertools::Itertools;
use std::path::PathBuf;

use crate::kak::io::KakouneIO;
use crate::os_io::OperatingSystemIO;
use crate::Checker;
use crate::{Dictionary, Repository};
use crate::{ProjectId, ProjectPath, RelativePath};

pub(crate) const SKYSPELL_PROJECT_OPT: &str = "skyspell_project";

pub(crate) struct Error {
    pos: (usize, usize),
    buffer: String,
    full_path: PathBuf,
    token: String,
}

pub(crate) struct KakouneChecker<D: Dictionary, R: Repository, S: OperatingSystemIO> {
    project: ProjectPath,
    project_id: ProjectId,
    pub(crate) dictionary: D,
    pub(crate) repository: R,
    errors: Vec<Error>,
    pub(crate) kakoune_io: KakouneIO<S>,
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
        let full_path = self.project.path().join(&path);
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
        &self.repository
    }

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn project(&self) -> &ProjectPath {
        &self.project
    }

    fn project_id(&self) -> ProjectId {
        self.project_id
    }
}

impl<D: Dictionary, R: Repository, S: OperatingSystemIO> KakouneChecker<D, R, S> {
    pub(crate) fn new(
        project: ProjectPath,
        dictionary: D,
        mut repository: R,
        kakoune_io: KakouneIO<S>,
    ) -> Result<Self> {
        let project_id = repository.ensure_project(&project)?;
        Ok(Self {
            project,
            project_id,
            dictionary,
            kakoune_io,
            repository,
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
        let project = &self.project;
        let errors_count = self.errors.len();
        self.print(&format!(
            "set global skyspell_error_count {}\n",
            errors_count
        ));
        match errors_count {
            0 => self.print(&format!(
                "echo -markup {}: {{green}}No spelling errors\n",
                project
            )),
            1 => self.print(&format!(
                "echo -markup {}: {{red}}1 spelling error\n",
                project
            )),
            n => self.print(&format!(
                "echo -markup {}: {{red}}{} spelling errors\n",
                project, n,
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

#[cfg(test)]
pub(crate) mod tests {

    use super::*;

    use tempdir::TempDir;

    use crate::kak::io::tests::new_fake_io;
    use crate::tests::FakeIO;
    use crate::tests::{FakeDictionary, FakeRepository};
    use crate::{ProjectPath, RelativePath};

    pub(crate) type FakeChecker = KakouneChecker<FakeDictionary, FakeRepository, FakeIO>;

    impl FakeChecker {
        pub(crate) fn get_output(self) -> String {
            self.kakoune_io.get_output()
        }

        pub(crate) fn ensure_path(&self, relative_name: &str) -> RelativePath {
            let project = self.project();
            let full_path = project.path().join(relative_name);
            std::fs::write(&full_path, "").unwrap();
            RelativePath::new(project, &full_path).unwrap()
        }
    }

    pub(crate) fn new_fake_checker(temp_dir: &TempDir) -> FakeChecker {
        let project = ProjectPath::open(temp_dir.path()).unwrap();
        let dictionary = FakeDictionary::new();
        let repository = FakeRepository::new();
        let fake_io = new_fake_io();
        KakouneChecker::new(project, dictionary, repository, fake_io).unwrap()
    }

    #[test]
    fn test_write_errors_in_spelling_buffer() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let mut checker = new_fake_checker(&temp_dir);
        let hello_js = checker.ensure_path("hello.js");
        checker.ensure_path("hello.js");
        checker
            .handle_error("foo", &hello_js, &(hello_js.to_string(), 2, 4))
            .unwrap();
        checker.write_spelling_buffer();
        let actual = checker.get_output();
        let expected = format!(
            "edit -scratch *spelling*
execute-keys \\% <ret> d i %{{{}/hello.js: 2.5,2.7 foo<ret>}} <esc> gg
",
            temp_dir.path().display()
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_write_errors_as_buffer_options() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let mut checker = new_fake_checker(&temp_dir);
        let foo_js = checker.ensure_path("foo.js");
        let bar_js = checker.ensure_path("bar.js");
        checker
            .handle_error("foo", &foo_js, &(foo_js.to_string(), 2, 4))
            .unwrap();

        checker
            .handle_error("bar", &foo_js, &(foo_js.to_string(), 3, 6))
            .unwrap();

        checker
            .handle_error("spam", &bar_js, &(bar_js.to_string(), 1, 5))
            .unwrap();

        let timestamp = 42;
        checker.write_ranges(timestamp);

        let actual = checker.get_output();
        let expected = "\
    set-option buffer=foo.js spell_errors 42 2.5+3|Error 3.7+3|Error \n\
    set-option buffer=bar.js spell_errors 42 1.6+4|Error \n";
        assert_eq!(actual, expected);
    }
}
