use std::collections::HashSet;
use std::path::Path;

use anyhow::{bail, Result};
use colored::*;

use crate::Interactor;
use crate::Repo;

pub struct Checker<I: Interactor, R: Repo> {
    interactor: I,
    repo: R,
    skipped: HashSet<String>,
}

impl<I: Interactor, R: Repo> Checker<I, R> {
    pub fn new(interactor: I, repo: R) -> Self {
        Self {
            interactor,
            repo,
            skipped: HashSet::new(),
        }
    }

    #[cfg(test)]
    fn interactor(&self) -> &I {
        &self.interactor
    }

    #[cfg(test)]
    fn repo(&self) -> &R {
        &self.repo
    }

    fn print_addition(&self, token: &str, location: &str) {
        println!("\n{}Added {} to {}\n", "=> ".blue(), token.blue(), location);
    }

    fn print_error(&self, details: &str) {
        eprintln!("{} {}", "Error:".red(), details);
    }

    pub fn handle_token(&mut self, path: &Path, pos: (usize, usize), token: &str) -> Result<()> {
        let token = token.to_lowercase();
        let ext = path.extension().and_then(|e| e.to_str());
        let file = path.to_str();
        if self.skipped.contains(&token) {
            // already skipped
            return Ok(());
        }
        let found = self.repo.lookup_word(&token.to_lowercase(), file, ext)?;
        if !found {
            self.handle_error(path, pos, &token)?;
        }
        Ok(())
    }

    // return false if error was *not* handled - will cause checker to exit with (1)
    fn handle_error(&mut self, path: &Path, pos: (usize, usize), error: &str) -> Result<()> {
        let (lineno, column) = pos;
        let prefix = format!("{}:{}:{}", path.display(), lineno, column);
        println!("{} {}", prefix.bold(), error.blue());
        let prompt = r#"What to do?

Add to (g)lobal ignore list
Add to ignore list for this (e)xtension
Add to ignore list for this (f)ile
(s)kip
(q)uit"#;

        loop {
            let letter = self.interactor.input_letter(prompt, "gefqs");
            match letter.as_ref() {
                "g" => return self.add_to_global_ignore(&error),
                "e" => {
                    let success = self.handle_ext(path, &error)?;
                    if success {
                        break;
                    }
                }
                "f" => {
                    let success = self.handle_file(path, &error)?;
                    if success {
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
        self.print_addition(error, "the global ignore list");
        Ok(())
    }

    fn handle_ext(&mut self, path: &Path, error: &str) -> Result<bool> {
        let os_ext = if let Some(os_ext) = path.extension() {
            os_ext
        } else {
            self.print_error(&format!("{} has no extension", path.display()));
            return Ok(false);
        };

        let ext = if let Some(s) = os_ext.to_str() {
            s
        } else {
            self.print_error(&format!("{} has a non-UTF-8 extension", path.display()));
            return Ok(false);
        };

        if !self.repo.known_extension(ext)? {
            self.repo.add_extension(ext)?;
        }

        self.repo.add_ignored_for_extension(error, ext)?;
        self.print_addition(
            error,
            &format!("the ignore list for extension {}", ext.bold()),
        );
        Ok(true)
    }

    fn handle_file(&mut self, path: &Path, error: &str) -> Result<bool> {
        let file_path = if let Some(s) = path.to_str() {
            s
        } else {
            self.print_error(&format!("{} has a non-UTF-8 extension", path.display()));
            return Ok(false);
        };

        if !self.repo.known_file(file_path)? {
            self.repo.add_file(file_path)?;
        }

        self.repo.add_ignored_for_file(error, file_path)?;
        self.print_addition(
            error,
            &format!("the ignore list for path {}", file_path.bold()),
        );
        Ok(true)
    }

    pub fn skipped(&self) -> bool {
        !self.skipped.is_empty()
    }
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

        let mut checker = Checker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("foo.txt"), (3, 2), "foo")
            .unwrap();

        checker.interactor().assert_empty();
        assert!(checker.repo().lookup_word("foo", None, None).unwrap());
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

        let mut checker = Checker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("hello.py"), (3, 2), "defaultdict")
            .unwrap();

        checker.interactor().assert_empty();
        assert!(checker
            .repo()
            .lookup_word("defaultdict", None, Some("py"))
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

        let mut checker = Checker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("hello.py"), (3, 2), "defaultdict")
            .unwrap();

        checker.interactor().assert_empty();
        assert!(checker
            .repo()
            .lookup_word("defaultdict", None, Some("py"))
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

        let mut checker = Checker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("poetry.lock"), (3, 2), "adbcdef")
            .unwrap();

        checker.interactor().assert_empty();
        assert!(checker
            .repo()
            .lookup_word("adbcdef", Some("poetry.lock"), Some("lock"))
            .unwrap());
    }

    #[test]
    /// Scenario:
    fn test_remember_skipped_tokens() {
        let mut fake_repo = FakeRepo::new();
        fake_repo.insert_good_words(&["hello", "world"]).unwrap();

        let fake_interactor = FakeInteractor::new();
        fake_interactor.push_text("s");

        let mut checker = Checker::new(fake_interactor, fake_repo);
        checker
            .handle_token(&Path::new("foo.py"), (3, 2), "foo")
            .unwrap();

        checker
            .handle_token(&Path::new("foo.py"), (5, 2), "foo")
            .unwrap();
    }
}
