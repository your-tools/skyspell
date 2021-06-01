use anyhow::Result;
use std::path::Path;

pub trait Repo {
    // Add the list of words to the global ignore list
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()>;

    // Add the extension to the list of known extensions
    fn add_extension(&mut self, ext: &str) -> Result<()>;
    // Add the file to the list of known full paths
    fn add_file(&mut self, full_path: &str) -> Result<()>;

    fn known_extension(&self, ext: &str) -> Result<bool>;
    fn known_file(&self, full_path: &str) -> Result<bool>;

    // Always skip this file name - to be used with Cargo.lock, yarn.lock
    // and the like
    fn skip_file_name(&mut self, file_name: &str) -> Result<()>;
    // Always skip this file path - when it's not actual source code
    fn skip_full_path(&mut self, full_path: &str) -> Result<()>;
    fn is_skipped(&self, path: &Path) -> Result<bool>;

    // Unskip this file name
    fn unskip_file_name(&mut self, file_name: &str) -> Result<()>;
    // Unskip this file path
    fn unskip_full_path(&mut self, full_path: &str) -> Result<()>;

    // Add word to the global ignore list
    fn add_ignored(&mut self, word: &str) -> Result<i32>;
    // Add word to the ignore list for the given extension
    fn add_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()>;
    // Add word to the ignore list for the given file
    fn add_ignored_for_file(&mut self, word: &str, file: &str) -> Result<()>;
    // Is the word in the global ignore list?
    fn is_ignored(&self, word: &str) -> Result<bool>;

    // Remove word from the global ignore list
    fn remove_ignored(&mut self, word: &str) -> Result<()>;
    // Remove word from the ignore list for the given extension
    fn remove_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()>;
    // Remove word from the ignore list for the given path
    fn remove_ignored_for_file(&mut self, word: &str, path: &str) -> Result<()>;

    // Lookup word in the repo. Return true if the word should be spell checked
    fn lookup_word(&self, word: &str, file: &Path) -> Result<bool>;
}

#[cfg(test)]
mod tests {
    use paste::paste;

    use crate::tests::FakeRepo;
    use crate::Db;

    use super::*;

    // Given an identifier and a block, generate a test
    // for each implementation of the Repo trait
    // (Db and FakeRepo)
    macro_rules! make_repo_tests {
        ($name:ident, ($repo:ident) => $test:block) => {
            paste! {
            fn $name(mut $repo: impl Repo) {
                $test
            }

            #[test]
            fn [<test_db_ $name>]() {
                let db = Db::connect(":memory:").unwrap();
                $name(db)
            }

            #[test]
            fn [<test_fake_repo_ $name>]() {
                let repo = FakeRepo::new();
                $name(repo)
            }
            }
        };
    }

    make_repo_tests!(known_extension, (repo) => {
        repo.add_extension("py").unwrap();
        assert!(repo.known_extension("py").unwrap());
        assert!(!repo.known_extension("rs").unwrap());
    });

    make_repo_tests!(known_file, (repo) => {
        repo.add_file("/path/to/foo").unwrap();
        assert!(repo.known_file("/path/to/foo").unwrap());
        assert!(!repo.known_file("/path/to/bar").unwrap());
    });

    make_repo_tests!(is_skipped, (repo) => {
        repo.skip_file_name("Cargo.lock").unwrap();
        repo.skip_full_path("/path/to/bar").unwrap();

        let path = Path::new("/path/to/Cargo.lock");
        assert!(repo.is_skipped(path).unwrap());

        let path = Path::new("/path/to/bar");
        assert!(repo.is_skipped(path).unwrap());

        let path = Path::new("/path/to/baz");
        assert!(!repo.is_skipped(path).unwrap());
    });

    make_repo_tests!(is_ignored, (repo) => {
        repo.add_ignored("foo").unwrap();

        assert!(repo.is_ignored("foo").unwrap());
        assert!(!repo.is_ignored("bar").unwrap());
    });

    make_repo_tests!(lookup_in_ignored_words, (repo) => {
        repo.add_ignored("foobar").unwrap();

        assert!(repo.lookup_word("foobar", &Path::new("-")).unwrap());
    });

    make_repo_tests!(lookup_in_ignored_extensions, (repo) => {
        repo.add_ignored("foobar").unwrap();
        repo.add_extension("py").unwrap();
        repo.add_ignored_for_extension("defaultdict", "py").unwrap();

        assert!(repo.lookup_word("defaultdict", &Path::new("foo.py")).unwrap());
    });

    make_repo_tests!(lookup_in_files, (repo) => {
        repo.add_file("path/to/poetry.lock").unwrap();
        repo.add_ignored_for_file("abcdef", "path/to/poetry.lock")
            .unwrap();

        assert!(repo
            .lookup_word("abcdef", &Path::new("path/to/poetry.lock"))
            .unwrap());
    });

    make_repo_tests!(lookup_in_skipped_file_names, (repo) => {
        repo.skip_file_name("poetry.lock").unwrap();

        assert!(repo.is_skipped(&Path::new("path/to/poetry.lock")).unwrap());
    });

    make_repo_tests!(remove_ignored, (repo) => {
        repo.add_ignored("foo").unwrap();
        assert!(repo.lookup_word("foo", Path::new("-'")).unwrap());

        repo.remove_ignored("foo").unwrap();
        assert!(!repo.lookup_word("foo", Path::new("-'")).unwrap());
    });

    make_repo_tests!(remove_ignored_for_ext, (repo) => {
        repo.add_extension("py").unwrap();
        repo.add_extension("rs").unwrap();
        repo.add_ignored_for_extension("foo", "py").unwrap();
        repo.add_ignored_for_extension("foo", "rs").unwrap();

        repo.remove_ignored_for_extension("foo", "py").unwrap();
        assert!(!repo.lookup_word("foo", Path::new("foo.py")).unwrap());
        assert!(repo.lookup_word("foo", Path::new("foo.rs")).unwrap());
    });

    make_repo_tests!(remove_ignored_for_file, (repo) => {
        repo.add_file("/path/to/one").unwrap();
        repo.add_file("/path/to/two").unwrap();
        repo.add_ignored_for_file("foo", "/path/to/one").unwrap();
        repo.add_ignored_for_file("foo", "/path/to/two").unwrap();

        repo.remove_ignored_for_file("foo", "/path/to/one").unwrap();
        assert!(!repo.lookup_word("foo", Path::new("/path/to/one")).unwrap());
        assert!(repo.lookup_word("foo", Path::new("/path/to/two")).unwrap());
    });

    make_repo_tests!(unskip_file_name, (repo) => {
        repo.skip_file_name("Cargo.lock").unwrap();
        let path = Path::new("/path/to/Cargo.lock");
        assert!(repo.is_skipped(path).unwrap());

        repo.unskip_file_name("Cargo.lock").unwrap();
        assert!(!repo.is_skipped(path).unwrap());
    });

    make_repo_tests!(unskip_file_path, (repo) => {
        repo.skip_full_path("/path/to/foo").unwrap();
        let path = Path::new("/path/to/foo");
        assert!(repo.is_skipped(path).unwrap());

        repo.unskip_full_path("/path/to/foo").unwrap();
        assert!(!repo.is_skipped(path).unwrap());
    });
}
