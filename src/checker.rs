use std::collections::HashSet;
use std::path::Path;

use anyhow::{bail, Result};
use colored::*;

use crate::Interactor;
use crate::Repo;

pub trait Checker {
    fn handle_token(&mut self, path: &Path, pos: (usize, usize), token: &str) -> Result<()>;
    fn success(&self) -> bool;
}

pub struct NonInteractiveChecker<R: Repo> {
    repo: R,
    errors_found: bool,
}

impl<R: Repo> NonInteractiveChecker<R> {
    pub fn new(repo: R) -> Self {
        Self {
            repo,
            errors_found: false,
        }
    }
}

impl<R: Repo> Checker for NonInteractiveChecker<R> {
    fn handle_token(&mut self, path: &Path, pos: (usize, usize), token: &str) -> Result<()> {
        let found = lookup_token(&self.repo, token, path)?;
        if !found {
            self.errors_found = true;
            print_unknown_token(token, path, pos);
        }
        Ok(())
    }

    fn success(&self) -> bool {
        !self.errors_found
    }
}

pub struct InteractiveChecker<I: Interactor, R: Repo> {
    interactor: I,
    repo: R,
    skipped: HashSet<String>,
}

impl<I: Interactor, R: Repo> Checker for InteractiveChecker<I, R> {
    fn success(&self) -> bool {
        self.skipped.is_empty()
    }

    fn handle_token(&mut self, path: &Path, pos: (usize, usize), token: &str) -> Result<()> {
        let found = lookup_token(&self.repo, token, path)?;
        if self.skipped.contains(token) {
            // already skipped
            return Ok(());
        }
        if !found {
            self.handle_error(path, pos, &token)?;
        }
        Ok(())
    }
}

impl<I: Interactor, R: Repo> InteractiveChecker<I, R> {
    pub fn new(interactor: I, repo: R) -> Self {
        Self {
            interactor,
            repo,
            skipped: HashSet::new(),
        }
    }

    #[allow(dead_code)]
    fn interactor(&self) -> &I {
        &self.interactor
    }

    #[allow(dead_code)]
    fn repo(&self) -> &R {
        &self.repo
    }

    // return false if error was *not* handled - will cause checker to exit with (1)
    fn handle_error(&mut self, path: &Path, pos: (usize, usize), error: &str) -> Result<()> {
        let (lineno, column) = pos;
        let prefix = format!("{}:{}:{}", path.display(), lineno, column);
        println!("{} {}", prefix.bold(), error.blue());
        let prompt = r#"What to do?

Add word to (g)lobal ignore list
Add word to ignore list for this (e)xtension
Add word to ignore list for this (f)ull path
Always skip this file (n)ame
(s)kip this error
(q)uit"#;

        loop {
            let letter = self.interactor.input_letter(prompt, "gefnqs");
            match letter.as_ref() {
                "g" => return self.add_to_global_ignore(&error),
                "e" => {
                    if self.handle_ext(path, &error)? {
                        break;
                    }
                }
                "f" => {
                    if self.handle_full_path(path, &error)? {
                        break;
                    }
                }
                "n" => {
                    if self.handle_file_name(path)? {
                        break;
                    }
                }
                "q" => {
                    bail!("Interrupted by user")
                }
                "s" => {
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

    fn add_to_global_ignore(&mut self, error: &str) -> Result<()> {
        self.repo.add_ignored(error)?;
        print_addition(error, "the global ignore list");
        Ok(())
    }

    fn handle_ext(&mut self, path: &Path, error: &str) -> Result<bool> {
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

        self.repo.add_extension(ext)?;

        self.repo.add_ignored_for_extension(error, ext)?;
        print_addition(
            error,
            &format!("the ignore list for extension {}", ext.bold()),
        );
        Ok(true)
    }

    fn handle_full_path(&mut self, path: &Path, error: &str) -> Result<bool> {
        let file_path = if let Some(s) = path.to_str() {
            s
        } else {
            print_error(&format!("{} has a non-UTF-8 extension", path.display()));
            return Ok(false);
        };

        self.repo.add_file(file_path)?;

        self.repo.add_ignored_for_file(error, file_path)?;
        print_addition(
            error,
            &format!("the ignore list for path {}", file_path.bold()),
        );
        Ok(true)
    }

    fn handle_file_name(&mut self, path: &Path) -> Result<bool> {
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
            "\n{}Added {} to the list of file names to skip\n",
            "=> ".blue(),
            file_name,
        );
        Ok(true)
    }

    pub fn skipped(&self) -> bool {
        !self.skipped.is_empty()
    }
}

fn print_addition(token: &str, location: &str) {
    println!("\n{}Added {} to {}\n", "=> ".blue(), token.blue(), location);
}

fn print_error(message: &str) {
    eprintln!("{} {}", "Error:".red(), message);
}

fn print_unknown_token(token: &str, path: &Path, pos: (usize, usize)) {
    let (line, column) = pos;
    let prefix = format!("{}:{}:{}", path.display(), line, column);
    println!("{} {}", prefix.bold(), token.blue());
}

fn lookup_token<R: Repo>(repo: &R, token: &str, path: &Path) -> Result<bool> {
    repo.lookup_word(&token.to_lowercase(), path)
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::{FakeInteractor, FakeRepo};

    #[test]
    /// Scenario:
    /// * call handle_error with 'foo'
    /// * add 'foo' to the global ignore list (aka: press 'g')
    ///
    /// Check that 'foo' is in the globally ignore list
    fn test_adding_to_ignore() {
        let mut fake_repo = FakeRepo::new();
        fake_repo.insert_good_words(&["hello", "world"]).unwrap();
        let fake_interactor = FakeInteractor::new();
        fake_interactor.push_text("g");

        let mut checker = InteractiveChecker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("foo.txt"), (3, 2), "foo")
            .unwrap();

        assert!(checker
            .repo()
            .lookup_word("foo", &Path::new("other.txt"))
            .unwrap());
    }

    #[test]
    /// Scenario:
    /// * call handle_error with 'foo' and path/to/yarn.lock
    /// * add 'yarn.lock' to the global ignore list (aka: press 'n')
    ///
    /// Check that 'foo' is in also ignored for other/path/to/yarn.lock
    fn test_adding_to_skipped() {
        let mut fake_repo = FakeRepo::new();
        fake_repo.insert_good_words(&["hello", "world"]).unwrap();
        let fake_interactor = FakeInteractor::new();
        fake_interactor.push_text("n");

        let mut checker = InteractiveChecker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("path/to/yarn.lock"), (3, 2), "foo")
            .unwrap();

        assert!(checker
            .repo()
            .lookup_word("foo", &Path::new("path/to/other/yarn.lock"))
            .unwrap());
    }

    #[test]
    /// Scenario:
    /// * no extensions known yet
    /// * call handle_token with 'defaultdict' error and a `.py` extension
    /// * press 'e'
    /// * confirm
    ///
    /// Check that 'foo' is ignored for the `py` extension
    fn test_adding_to_new_ext() {
        let mut fake_repo = FakeRepo::new();
        fake_repo.insert_good_words(&["hello", "world"]).unwrap();
        let fake_interactor = FakeInteractor::new();
        fake_interactor.push_text("e");

        let mut checker = InteractiveChecker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("hello.py"), (3, 2), "defaultdict")
            .unwrap();

        assert!(checker
            .repo()
            .lookup_word("defaultdict", &Path::new("hello.py"))
            .unwrap());
    }

    #[test]
    /// Scenario:
    /// * py extension is known
    /// * call handle_token with 'defaultdict' error and a `.py` extension
    /// * press 'e'
    ///
    /// Check that 'foo' is ignored for the `py` extension
    fn test_adding_to_existing_ext() {
        let mut fake_repo = FakeRepo::new();
        fake_repo.insert_good_words(&["hello", "world"]).unwrap();
        fake_repo.add_extension("py").unwrap();

        let fake_interactor = FakeInteractor::new();
        fake_interactor.push_text("e");

        let mut checker = InteractiveChecker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("hello.py"), (3, 2), "defaultdict")
            .unwrap();

        assert!(checker
            .repo()
            .lookup_word("defaultdict", Path::new("hello.py"))
            .unwrap());
    }

    #[test]
    /// Scenario:
    /// * poetry.lock file is known
    /// * call handle_token with 'abcdef' error ,  `lock` extension and a `poetry.lock` file
    /// * press 'f'
    ///
    /// Check that 'adbced' is ignored for the `poetry.lock` file
    fn test_adding_to_existing_file() {
        let mut fake_repo = FakeRepo::new();
        fake_repo.insert_good_words(&["hello", "world"]).unwrap();
        fake_repo.add_extension("py").unwrap();
        fake_repo.add_file("poetry.lock").unwrap();

        let fake_interactor = FakeInteractor::new();
        fake_interactor.push_text("f");

        let mut checker = InteractiveChecker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("poetry.lock"), (3, 2), "adbcdef")
            .unwrap();

        assert!(checker
            .repo()
            .lookup_word("adbcdef", &Path::new("poetry.lock"))
            .unwrap());
    }

    #[test]
    /// Scenario:
    fn test_remember_skipped_tokens() {
        let mut fake_repo = FakeRepo::new();
        fake_repo.insert_good_words(&["hello", "world"]).unwrap();

        let fake_interactor = FakeInteractor::new();
        fake_interactor.push_text("s");

        let mut checker = InteractiveChecker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("foo.py"), (3, 2), "foo")
            .unwrap();

        checker
            .handle_token(&Path::new("foo.py"), (5, 2), "foo")
            .unwrap();
    }

    #[test]
    /// Scenario:
    fn test_remember_extensions() {
        let mut fake_repo = FakeRepo::new();
        fake_repo.insert_good_words(&["hello", "world"]).unwrap();

        let fake_interactor = FakeInteractor::new();
        fake_interactor.push_text("e");

        let mut checker = InteractiveChecker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("foo.py"), (3, 2), "abstractmethod")
            .unwrap();

        checker
            .handle_token(&Path::new("foo.py"), (10, 2), "abstractmethod")
            .unwrap();
    }
}
