use std::collections::HashSet;

use anyhow::{bail, Result};
use colored::*;

use crate::repository::ProjectId;
use crate::Checker;
use crate::Dictionary;
use crate::Interactor;
use crate::Repository;
use crate::{Project, RelativePath};

pub(crate) struct InteractiveChecker<I: Interactor, D: Dictionary, R: Repository> {
    project: Project,
    project_id: ProjectId,
    interactor: I,
    dictionary: D,
    repository: R,
    skipped: HashSet<String>,
}

impl<I: Interactor, D: Dictionary, R: Repository> Checker for InteractiveChecker<I, D, R> {
    // line, column
    type Context = (usize, usize);

    fn success(&self) -> Result<()> {
        if !self.skipped.is_empty() {
            bail!("Some errors were skipped")
        }
        Ok(())
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn repository(&self) -> &dyn Repository {
        &self.repository
    }

    fn handle_error(
        &mut self,
        error: &str,
        path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        let &(line, column) = context;
        // The list of skipped paths may have changed
        if self.should_skip(path)? {
            return Ok(());
        }
        if self.skipped.contains(error) {
            return Ok(());
        }
        self.on_error(path, (line, column), error)
    }
}

impl<I: Interactor, D: Dictionary, R: Repository> InteractiveChecker<I, D, R> {
    pub(crate) fn new(
        project: Project,
        interactor: I,
        dictionary: D,
        mut repository: R,
    ) -> Result<Self> {
        let project_id = repository.ensure_project(&project)?;
        Ok(Self {
            project,
            project_id,
            dictionary,
            interactor,
            repository,
            skipped: HashSet::new(),
        })
    }

    fn print_addition(token: &str, location: &str) {
        println!("\n{}Added {} to {}\n", "=> ".blue(), token.blue(), location);
    }

    fn print_error(message: &str) {
        eprintln!("{} {}", "Error:".red(), message);
    }

    fn on_error(&mut self, path: &RelativePath, pos: (usize, usize), error: &str) -> Result<()> {
        let (lineno, column) = pos;
        let prefix = format!("{}:{}:{}", path, lineno, column);
        println!("{} {}", prefix.bold(), error.blue());
        let prompt = r#"What to do?
a : Add word to global ignore list
e : Add word to ignore list for this extension
p : Add word to ignore list for the current project
f : Add word to ignore list for the current file
n : Always skip this file name
s : Always skip this file path
x : Skip this error
q : Quit
> "#;

        loop {
            let letter = self.interactor.input_letter(prompt, "aepfnsxq");
            match letter.as_ref() {
                "a" => return self.on_global_ignore(error),
                "e" => {
                    if self.on_extension(path, error)? {
                        break;
                    }
                }
                "p" => {
                    if self.on_project_ignore(error)? {
                        break;
                    }
                }
                "f" => {
                    if self.on_file_ignore(error, path)? {
                        break;
                    }
                }
                "n" => {
                    if self.on_file_name_skip(path)? {
                        break;
                    }
                }
                "s" => {
                    if self.on_project_file_skip(path)? {
                        break;
                    }
                }
                "q" => {
                    bail!("Interrupted by user")
                }
                "x" => {
                    self.skipped.insert(error.to_string());
                    break;
                }
                _ => {
                    unreachable!()
                }
            }
        }
        Ok(())
    }

    fn on_global_ignore(&mut self, error: &str) -> Result<()> {
        self.repository.ignore(error)?;
        Self::print_addition(error, "the global ignore list");
        Ok(())
    }

    fn on_extension(&mut self, relative_path: &RelativePath, error: &str) -> Result<bool> {
        let extension = match relative_path.extension() {
            None => {
                Self::print_error(&format!("{} has no extension", relative_path));
                return Ok(false);
            }
            Some(e) => e,
        };

        self.repository.ignore_for_extension(error, &extension)?;
        Self::print_addition(
            error,
            &format!("the ignore list for extension '.{}'", extension.bold()),
        );
        Ok(true)
    }

    fn on_project_ignore(&mut self, error: &str) -> Result<bool> {
        self.repository.ignore_for_project(error, self.project_id)?;
        Self::print_addition(
            error,
            &format!("the ignore list for project '{}'", &self.project),
        );
        Ok(true)
    }

    fn on_file_ignore(&mut self, error: &str, relative_path: &RelativePath) -> Result<bool> {
        self.repository
            .ignore_for_path(error, &self.project, relative_path)?;

        Self::print_addition(
            error,
            &format!("the ignore list for path '{}'", relative_path),
        );
        Ok(true)
    }

    fn on_file_name_skip(&mut self, relative_path: &RelativePath) -> Result<bool> {
        let file_name = match relative_path.file_name() {
            None => {
                Self::print_error(&format!("{} has no file name", relative_path));
                return Ok(false);
            }
            Some(r) => r,
        };

        self.repository.skip_file_name(&file_name)?;

        println!(
            "\n{}Added '{}' to the list of file names to skip\n",
            "=> ".blue(),
            file_name,
        );
        Ok(true)
    }

    fn on_project_file_skip(&mut self, relative_path: &RelativePath) -> Result<bool> {
        self.repository.skip_path(&self.project, relative_path)?;
        println!(
            "\n{}Added '{}' to the list of files to skip for project: '{}'\n",
            "=> ".blue(),
            relative_path,
            &self.project.as_str().bold(),
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use tempdir::TempDir;

    use crate::tests::{FakeDictionary, FakeInteractor, FakeRepository};

    type TestChecker = InteractiveChecker<FakeInteractor, FakeDictionary, FakeRepository>;

    struct TestApp {
        checker: TestChecker,
    }

    impl TestApp {
        fn new(temp_dir: &TempDir) -> Self {
            let interactor = FakeInteractor::new();
            let dictionary = FakeDictionary::new();
            let repository = FakeRepository::new();
            let project = Project::open(temp_dir.path()).unwrap();
            let checker = TestChecker::new(project, interactor, dictionary, repository).unwrap();
            Self { checker }
        }

        fn add_known(&mut self, words: &[&str]) {
            for word in words.iter() {
                self.checker.dictionary.add_known(word);
            }
        }

        fn push_text(&mut self, answer: &str) {
            self.checker.interactor.push_text(answer)
        }

        fn to_relative_path(&self, path: &str) -> RelativePath {
            let project = self.checker.project();
            let path = project.path().join(path);
            RelativePath::new(project, &path).unwrap()
        }

        fn handle_token(&mut self, token: &str, relative_name: &str) {
            let project = self.checker.project();
            let full_path = project.path().join(relative_name);
            std::fs::write(&full_path, "").unwrap();
            let relative_path = self.to_relative_path(relative_name);
            let context = &(3, 42);
            self.checker
                .handle_token(token, &relative_path, context)
                .unwrap()
        }

        fn is_ignored(&self, word: &str) -> bool {
            self.checker.repository.is_ignored(word).unwrap()
        }

        fn is_skipped_file_name(&self, file_name: &str) -> bool {
            self.checker
                .repository
                .is_skipped_file_name(file_name)
                .unwrap()
        }

        fn is_skipped_path(&self, relative_name: &str) -> bool {
            let project = self.checker.project();
            let relative_path = self.to_relative_path(relative_name);
            self.checker
                .repository
                .is_skipped_path(project, &relative_path)
                .unwrap()
        }

        fn is_ignored_for_extension(&self, word: &str, extension: &str) -> bool {
            self.checker
                .repository
                .is_ignored_for_extension(word, extension)
                .unwrap()
        }

        fn is_ignored_for_project(&self, word: &str) -> bool {
            let project_id = self.checker.project_id;
            self.checker
                .repository
                .is_ignored_for_project(word, project_id)
                .unwrap()
        }

        fn is_ignored_for_path(&self, word: &str, relative_name: &str) -> bool {
            let project = self.checker.project();
            let relative_path = self.to_relative_path(relative_name);
            self.checker
                .repository
                .is_ignored_for_path(word, project, &relative_path)
                .unwrap()
        }

        fn end(&self) {
            if !self.checker.interactor.is_empty() {
                panic!("Not all answered consumed by the test");
            }
        }
    }

    #[test]
    fn test_adding_token_to_global_ignore() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.add_known(&["hello", "world"]);
        app.push_text("a");

        app.handle_token("foo", "foo.txt");

        assert!(app.is_ignored("foo"));
        app.handle_token("foo", "other.ext");

        app.end();
    }

    #[test]
    fn test_adding_token_to_extension() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.add_known(&["hello", "world"]);
        app.push_text("e");

        app.handle_token("defaultdict", "hello.py");

        assert!(app.is_ignored_for_extension("defaultdict", "py"));
        app.handle_token("defaultdict", "bar.py");

        app.end();
    }

    #[test]
    fn test_adding_token_to_project() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.push_text("p");

        app.handle_token("foo", "foo.py");

        assert!(app.is_ignored_for_project("foo"));
        app.handle_token("foo", "foo.py");

        app.end()
    }

    #[test]
    fn test_ignore_token_to_project_file() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.push_text("f");

        app.handle_token("foo", "foo.py");

        assert!(app.is_ignored_for_path("foo", "foo.py"));
        app.handle_token("foo", "foo.py");

        app.end()
    }

    #[test]
    fn test_adding_to_skipped_file_names() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.add_known(&["hello", "world"]);
        app.push_text("n");

        app.handle_token("foo", "yarn.lock");

        assert!(app.is_skipped_file_name("yarn.lock"));
        app.handle_token("bar", "yarn.lock");

        app.end();
    }

    #[test]
    fn test_adding_to_skipped_paths() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.push_text("s");

        app.handle_token("foo", "foo.py");

        assert!(app.is_skipped_path("foo.py"));
        app.handle_token("bar", "foo.py");

        app.end();
    }

    #[test]
    fn test_remember_skipped_tokens() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.add_known(&["hello", "world"]);
        app.push_text("x");

        app.handle_token("foo", "foo.py");
        app.handle_token("foo", "foo.py");

        app.end();
    }
}
