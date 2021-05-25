use std::collections::HashSet;
use std::path::Path;

use anyhow::{bail, Result};
use colored::*;

use crate::Dictionary;
use crate::Interactor;
use crate::Repo;

pub trait Checker {
    fn is_skipped(&self, path: &Path) -> Result<bool>;
    fn handle_token(&mut self, path: &Path, pos: (usize, usize), token: &str) -> Result<()>;
    fn success(&self) -> bool;
}

pub struct NonInteractiveChecker<D: Dictionary, R: Repo> {
    dictionary: D,
    repo: R,
    errors_found: bool,
}

impl<D: Dictionary, R: Repo> NonInteractiveChecker<D, R> {
    pub fn new(dictionary: D, repo: R) -> Self {
        Self {
            dictionary,
            repo,
            errors_found: false,
        }
    }
}

impl<D: Dictionary, R: Repo> Checker for NonInteractiveChecker<D, R> {
    fn handle_token(&mut self, path: &Path, pos: (usize, usize), token: &str) -> Result<()> {
        let found = lookup_token(&self.dictionary, &self.repo, token, path)?;
        if !found {
            self.errors_found = true;
            print_unknown_token(token, path, pos);
        }
        Ok(())
    }

    fn is_skipped(&self, path: &Path) -> Result<bool> {
        self.repo.is_skipped(path)
    }

    fn success(&self) -> bool {
        !self.errors_found
    }
}

pub struct InteractiveChecker<I: Interactor, D: Dictionary, R: Repo> {
    interactor: I,
    dictionary: D,
    repo: R,
    skipped: HashSet<String>,
}

impl<I: Interactor, D: Dictionary, R: Repo> Checker for InteractiveChecker<I, D, R> {
    fn success(&self) -> bool {
        self.skipped.is_empty()
    }

    fn handle_token(&mut self, path: &Path, pos: (usize, usize), token: &str) -> Result<()> {
        let found = lookup_token(&self.dictionary, &self.repo, token, path)?;
        if self.skipped.contains(token) {
            // already skipped
            return Ok(());
        }
        if !found {
            self.handle_error(path, pos, &token)?;
        }
        Ok(())
    }

    fn is_skipped(&self, path: &Path) -> Result<bool> {
        self.repo.is_skipped(path)
    }
}

impl<I: Interactor, D: Dictionary, R: Repo> InteractiveChecker<I, D, R> {
    pub fn new(interactor: I, dictionary: D, repo: R) -> Self {
        Self {
            dictionary,
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

    fn handle_error(&mut self, path: &Path, pos: (usize, usize), error: &str) -> Result<()> {
        if self.is_skipped(path)? {
            return Ok(());
        }

        let (lineno, column) = pos;
        let prefix = format!("{}:{}:{}", path.display(), lineno, column);
        println!("{} {}", prefix.bold(), error.blue());
        let prompt = r#"What to do?

Add word to (g)lobal ignore list
Add word to ignore list for this (e)xtension
Add word to ignore list for this (f)ull path
Always skip this file (n)ame
Always skip this file (p)ath
(s)kip this error
(q)uit"#;

        loop {
            let letter = self.interactor.input_letter(prompt, "gefnqps");
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
                    if self.handle_file_name_skip(path)? {
                        break;
                    }
                }
                "p" => {
                    if self.handle_full_path_skip(path)? {
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

        if !self.repo.known_extension(ext)? {
            self.repo.add_extension(ext)?;
        }

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

        if !self.repo.known_file(file_path)? {
            self.repo.add_file(file_path)?;
        }

        self.repo.add_ignored_for_file(error, file_path)?;
        print_addition(
            error,
            &format!("the ignore list for path {}", file_path.bold()),
        );
        Ok(true)
    }

    fn handle_file_name_skip(&mut self, path: &Path) -> Result<bool> {
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

    fn handle_full_path_skip(&mut self, path: &Path) -> Result<bool> {
        let full_path = if let Some(s) = path.to_str() {
            s
        } else {
            print_error(&format!("{} is not valid UTF-8", path.display()));
            return Ok(false);
        };

        self.repo.skip_full_path(full_path)?;

        println!(
            "\n{}Added {} to the list of file paths to skip\n",
            "=> ".blue(),
            full_path,
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

fn lookup_token<D: Dictionary, R: Repo>(
    dictionary: &D,
    repo: &R,
    token: &str,
    path: &Path,
) -> Result<bool> {
    let is_ignored = repo.lookup_word(&token, path)?;
    if is_ignored {
        return Ok(true);
    } else {
        dictionary.check(token)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::tests::{FakeDictionary, FakeInteractor, FakeRepo};

    #[derive(Default)]
    struct TestApp {
        pub dictionary: FakeDictionary,
        pub repo: FakeRepo,
        pub interactor: FakeInteractor,
    }

    impl TestApp {
        fn checker(self) -> InteractiveChecker<impl Interactor, impl Dictionary, impl Repo> {
            InteractiveChecker::new(self.interactor, self.dictionary, self.repo)
        }

        fn new() -> Self {
            Default::default()
        }

        fn add_known(&mut self, words: &[&str]) {
            for word in words.iter() {
                self.dictionary.add_known(word);
            }
        }

        fn add_extension(&mut self, ext: &str) {
            self.repo.add_extension(ext).unwrap();
        }

        fn add_file(&mut self, file: &str) {
            self.repo.add_file(file).unwrap();
        }

        fn push_text(&mut self, answer: &str) {
            self.interactor.push_text(answer)
        }
    }

    #[test]
    /// Scenario:
    /// * call handle_error with 'foo'
    /// * add 'foo' to the global ignore list (aka: press 'g')
    ///
    /// Check that 'foo' is in the globally ignore list
    fn test_adding_to_ignore() {
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("g");
        let mut checker = app.checker();

        checker
            .handle_token(&Path::new("foo.txt"), (3, 2), "foo")
            .unwrap();

        assert!(checker
            .repo
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
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("n");
        let mut checker = app.checker();

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
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("e");
        let mut checker = app.checker();

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
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.add_extension("py");
        app.push_text("e");
        let mut checker = app.checker();

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
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.add_extension("py");
        app.add_file("poetry.lock");
        app.push_text("f");
        let mut checker = app.checker();

        checker
            .handle_token(&Path::new("poetry.lock"), (3, 2), "adbcdef")
            .unwrap();

        assert!(checker
            .repo()
            .lookup_word("adbcdef", &Path::new("poetry.lock"))
            .unwrap());
    }

    /// Scenario:
    /// * call handle_token with 'foo' error
    /// * press 's' - 'foo' token is skipped
    /// * call handle_token again
    /// * check that no more interaction took place
    ///   (this is done by FakeInteractor::drop, by the way)
    #[test]
    fn test_remember_skipped_tokens() {
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("s");
        let mut checker = app.checker();

        checker
            .handle_token(&Path::new("foo.py"), (3, 2), "foo")
            .unwrap();

        checker
            .handle_token(&Path::new("foo.py"), (5, 2), "foo")
            .unwrap();
    }

    /// Scenario:
    /// * 'py' extension is not known
    /// * call handle_token with 'foo' error
    /// * press 'e' - 'foo' token is added to the ignore list for '.py' extensions
    /// * call handle_token again
    /// * check that no more interaction took place
    #[test]
    fn test_remember_extensions() {
        let mut app = TestApp::new();
        app.add_known(&["hello", "world"]);
        app.push_text("e");
        let mut checker = app.checker();

        checker
            .handle_token(&Path::new("foo.py"), (3, 2), "abstractmethod")
            .unwrap();

        checker
            .handle_token(&Path::new("foo.py"), (10, 2), "abstractmethod")
            .unwrap();
    }
}
