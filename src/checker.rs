use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Result};
use colored::*;

use crate::Dictionary;
use crate::Interactor;
use crate::Repository;

pub(crate) trait Checker {
    type Context;

    fn handle_error(&mut self, error: &str, path: &Path, context: &Self::Context) -> Result<()>;

    fn success(&self) -> bool;
    fn repo_mut(&mut self) -> &mut dyn Repository;
    fn repo(&self) -> &dyn Repository;
    fn dictionary(&self) -> &dyn Dictionary;

    fn set_project_path(&mut self, path: &Path);
    fn project_path(&self) -> Option<&Path>;

    fn should_skip(&self, path: &Path) -> Result<bool> {
        let repo = self.repo();
        let project_path = self.project_path();
        repo.should_skip(project_path, path)
    }

    fn handle_token(&mut self, token: &str, path: &Path, context: &Self::Context) -> Result<()> {
        let token = &token.to_lowercase();
        if self.should_skip(path)? {
            return Ok(());
        }
        let project_path = self.project_path();
        let dictionary = self.dictionary();
        let in_dict = dictionary.check(token)?;
        if in_dict {
            return Ok(());
        }
        let repo = self.repo();
        let should_ignore = repo.should_ignore(&token, project_path, path)?;
        if !should_ignore {
            self.handle_error(token, path, context)?
        }
        Ok(())
    }

    fn ensure_project(&mut self, path: &Path) -> Result<()> {
        let repo = self.repo_mut();
        if !repo.project_exists(path)? {
            repo.new_project(path)?;
        }
        self.set_project_path(path);
        Ok(())
    }
}

pub(crate) struct NonInteractiveChecker<D: Dictionary, R: Repository> {
    dictionary: D,
    repo: R,
    errors_found: bool,
    project_path: Option<PathBuf>,
}

impl<D: Dictionary, R: Repository> NonInteractiveChecker<D, R> {
    pub(crate) fn new(dictionary: D, repo: R) -> Self {
        Self {
            dictionary,
            repo,
            errors_found: false,
            project_path: None,
        }
    }
}

impl<D: Dictionary, R: Repository> Checker for NonInteractiveChecker<D, R> {
    // line, column
    type Context = (usize, usize);

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn handle_error(&mut self, token: &str, path: &Path, context: &Self::Context) -> Result<()> {
        let &(line, column) = context;
        self.errors_found = true;
        print_unknown_token(token, path, line, column);
        Ok(())
    }

    fn success(&self) -> bool {
        !self.errors_found
    }

    fn project_path(&self) -> Option<&Path> {
        self.project_path.as_ref().map(|x| x.as_ref())
    }

    fn set_project_path(&mut self, path: &Path) {
        self.project_path = Some(path.to_path_buf())
    }

    fn repo_mut(&mut self) -> &mut dyn Repository {
        &mut self.repo
    }

    fn repo(&self) -> &dyn Repository {
        &self.repo
    }
}

pub(crate) struct InteractiveChecker<I: Interactor, D: Dictionary, R: Repository> {
    interactor: I,
    dictionary: D,
    repo: R,
    project_path: Option<PathBuf>,
    skipped: HashSet<String>,
}

impl<I: Interactor, D: Dictionary, R: Repository> Checker for InteractiveChecker<I, D, R> {
    // line, column
    type Context = (usize, usize);

    fn success(&self) -> bool {
        self.skipped.is_empty()
    }

    fn project_path(&self) -> Option<&Path> {
        self.project_path.as_ref().map(|x| x.as_ref())
    }

    fn set_project_path(&mut self, path: &Path) {
        self.project_path = Some(path.to_path_buf());
    }

    fn repo_mut(&mut self) -> &mut dyn Repository {
        &mut self.repo
    }

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn repo(&self) -> &dyn Repository {
        &self.repo
    }

    fn handle_error(&mut self, error: &str, path: &Path, context: &Self::Context) -> Result<()> {
        let &(line, column) = context;
        if self.skipped.contains(error) {
            return Ok(());
        }
        self.on_error(path, (line, column), error)
    }
}

impl<I: Interactor, D: Dictionary, R: Repository> InteractiveChecker<I, D, R> {
    pub(crate) fn new(interactor: I, dictionary: D, repo: R) -> Self {
        Self {
            dictionary,
            interactor,
            repo,
            project_path: None,
            skipped: HashSet::new(),
        }
    }

    fn on_error(&mut self, path: &Path, pos: (usize, usize), error: &str) -> Result<()> {
        let (lineno, column) = pos;
        let prefix = format!("{}:{}:{}", path.display(), lineno, column);
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
                "a" => return self.on_global_ignore(&error),
                "e" => {
                    if self.on_extension(path, &error)? {
                        break;
                    }
                }
                "p" => {
                    if self.on_project_ignore(&error)? {
                        break;
                    }
                }
                "f" => {
                    if self.on_file_ignore(&error, path)? {
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
        self.repo.ignore(error)?;
        print_addition(error, "the global ignore list");
        Ok(())
    }

    fn on_extension(&mut self, path: &Path, error: &str) -> Result<bool> {
        let os_ext = if let Some(os_ext) = path.extension() {
            os_ext
        } else {
            print_error(&format!("{} has no extension", path.display()));
            return Ok(false);
        };

        let ext = if let Some(s) = os_ext.to_str() {
            s
        } else {
            print_error(&format!("{} has a non-UTF-8 extension", path.display()));
            return Ok(false);
        };

        self.repo.ignore_for_extension(error, ext)?;
        print_addition(
            error,
            &format!("the ignore list for extension '.{}'", ext.bold()),
        );
        Ok(true)
    }

    fn on_project_ignore(&mut self, error: &str) -> Result<bool> {
        Ok(match self.project_path.as_ref() {
            None => {
                print_error("No project was set\n");
                false
            }
            Some(p) => {
                self.repo.ignore_for_project(error, p)?;
                print_addition(
                    error,
                    &format!("the ignore list for project '{}'", p.display()),
                );
                true
            }
        })
    }

    fn on_file_ignore(&mut self, error: &str, path: &Path) -> Result<bool> {
        Ok(match self.project_path.as_ref() {
            None => {
                print_error("No project was set\n");
                false
            }
            Some(project_path) => {
                let relative_path = pathdiff::diff_paths(path, project_path).ok_or_else(|| {
                    anyhow!(
                        "Could not build relative path from {} to {}",
                        path.display(),
                        project_path.display()
                    )
                })?;
                self.repo
                    .ignore_for_path(error, project_path, &relative_path)?;
                print_addition(
                    error,
                    &format!("the ignore list for path '{}'", path.display()),
                );
                true
            }
        })
    }

    fn on_file_name_skip(&mut self, path: &Path) -> Result<bool> {
        let file_name = if let Some(s) = path.file_name() {
            s
        } else {
            print_error(&format!("{} has no file name", path.display()));
            return Ok(false);
        };

        let file_name = if let Some(s) = file_name.to_str() {
            s
        } else {
            print_error(&format!("{} has a non-UTF-8 file name", path.display()));
            return Ok(false);
        };

        self.repo.skip_file_name(file_name)?;

        println!(
            "\n{}Added '{}' to the list of file names to skip\n",
            "=> ".blue(),
            file_name,
        );
        Ok(true)
    }

    fn on_project_file_skip(&mut self, path: &Path) -> Result<bool> {
        let project_path = match self.project_path.as_ref() {
            None => {
                eprintln!("No project was set");
                return Ok(false);
            }
            Some(p) => p,
        };

        self.repo.skip_path(project_path, path)?;
        println!(
            "\n{}Added '{:?}' to the list of files to skip for project: '{}'\n",
            "=> ".blue(),
            path.display(),
            project_path.to_string_lossy().bold(),
        );
        Ok(true)
    }
}

fn print_addition(token: &str, location: &str) {
    println!("\n{}Added {} to {}\n", "=> ".blue(), token.blue(), location);
}

fn print_error(message: &str) {
    eprintln!("{} {}", "Error:".red(), message);
}

fn print_unknown_token(token: &str, path: &Path, line: usize, column: usize) {
    let prefix = format!("{}:{}:{}", path.display(), line, column);
    println!("{} {}", prefix.bold(), token.blue());
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::{FakeDictionary, FakeInteractor, FakeRepository};

    type TestChecker = InteractiveChecker<FakeInteractor, FakeDictionary, FakeRepository>;

    struct TestApp {
        checker: TestChecker,
    }

    impl TestApp {
        fn new() -> Self {
            let interactor = FakeInteractor::new();
            let dictionary = FakeDictionary::new();
            let repository = FakeRepository::new();
            let checker = TestChecker::new(interactor, dictionary, repository);
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

        fn handle_token(&mut self, token: &str, path: &str) {
            let path = &Path::new(path);
            let context = &(3, 42);
            self.checker.handle_token(token, path, context).unwrap()
        }

        fn ensure_project(&mut self, project_path: &Path) {
            self.checker.ensure_project(project_path).unwrap()
        }

        fn should_ignore(
            &self,
            token: &str,
            project_path: Option<&Path>,
            relative_path: &Path,
        ) -> bool {
            self.checker
                .repo
                .should_ignore(token, project_path, relative_path)
                .unwrap()
        }

        fn should_skip(&self, project_path: Option<&Path>, relative_path: &Path) -> bool {
            self.checker
                .repo
                .should_skip(project_path, relative_path)
                .unwrap()
        }

        fn end(&self) {
            if !self.checker.interactor.is_empty() {
                panic!("Not all answered consumed by the test");
            }
        }
    }

    #[test]
    /// Scenario:
    /// * call handle_token with 'foo'
    /// * add 'foo' to the global ignore list (aka: press 'g')
    ///
    /// Check that 'foo' is in the globally ignore list
    fn test_adding_to_ignore() {
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("a");

        app.handle_token("foo", "foo.txt");

        assert!(app.should_ignore("foo", None, &Path::new("other.txt")));

        app.end();
    }

    #[test]
    /// Scenario:
    /// * call handle_token with 'foo' and path/to/yarn.lock
    /// * add 'yarn.lock' to the file names to skip (aka: press 'n')
    ///
    /// Check that 'foo' is in also ignored for other/path/to/yarn.lock
    fn test_adding_to_skipped() {
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("n");

        app.handle_token("foo", "path/to/yarn.lock");

        assert!(app.should_skip(None, &Path::new("path/to/other/yarn.lock")));

        app.end();
    }

    #[test]
    /// Scenario:
    /// * call handle_token with 'defaultdict' error and a `.py` extension
    /// * press 'e'
    /// * confirm
    ///
    /// Check that 'defaultdict' is ignored for the `py` extension
    fn test_adding_to_extension() {
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("e");

        app.handle_token("defaultdict", "hello.py");

        assert!(app.should_ignore("defaultdict", None, &Path::new("hello.py")));

        app.handle_token("defaultdict", "hello.py");

        app.end();
    }

    /// Scenario:
    /// * call handle_token with 'foo' error
    /// * press 'x' - 'foo' token is skipped
    /// * call handle_token again
    /// * check that no more interaction took place
    ///   (this is done by FakeInteractor::drop, by the way)
    #[test]
    fn test_remember_skipped_tokens() {
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("x");

        app.handle_token("foo", "foo.py");
        app.handle_token("foo", "foo.py");

        app.end();
    }

    /// Scenario:
    /// * call handle_token with 'abstractmethod' error
    /// * press 'e' - 'abstractmethod' token is added to the ignore list for '.py' extensions
    /// * call handle_token again
    /// * check that no more interaction took place
    #[test]
    fn test_remember_extensions() {
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("e");

        app.handle_token("abstractmethod", "foo.py");
        app.handle_token("abstractmethod", "foo.py");

        app.end();
    }

    /// Scenario:
    /// * call handle_token with 'foo' error
    /// * press 'p' - 'foo' token is added to the ignore list for the current project
    #[test]
    fn test_ignore_token_for_project() {
        let mut app = TestApp::new();
        app.push_text("p");

        app.ensure_project(Path::new("/path/to/project"));

        app.handle_token("foo", "/path/to/project/foo.py");

        assert!(app.should_ignore(
            "foo",
            Some(&Path::new("/path/to/project")),
            &Path::new("hello.py")
        ));

        app.handle_token("foo", "/path/to/project/foo.py");

        app.end()
    }

    /// Scenario:
    /// * call handle_token with 'foo' error
    /// * press 'p' - 'foo' token is added to the ignore list for the current project
    #[test]
    fn test_ignore_for_project_path() {
        let mut app = TestApp::new();
        app.push_text("s");

        app.ensure_project(Path::new("/path/to/project"));

        app.handle_token("foo", "/path/to/project/foo.py");

        assert!(app.should_skip(
            Some(&Path::new("/path/to/project")),
            &Path::new("/path/to/project/foo.py")
        ));

        app.end();
    }
}
