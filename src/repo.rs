use anyhow::Result;
use std::path::Path;

pub trait Repo {
    // Add the list of words to the good words
    fn insert_good_words(&mut self, words: &[&str]) -> Result<()>;
    // Add the list of words to the global ignore list
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()>;

    // Add the extension to the list of known extensions
    fn add_extension(&mut self, ext: &str) -> Result<()>;
    // Add the file to the list of known full paths
    fn add_file(&mut self, full_path: &str) -> Result<()>;

    // Always skip this file name - to be used with Cargo.lock, yarn.lock
    // and the like
    fn skip_file_name(&mut self, filename: &str) -> Result<()>;
    // Always skip this file path - when it's not actual source code
    fn skip_full_path(&mut self, full_path: &str) -> Result<()>;
    fn is_skipped(&self, path: &Path) -> Result<bool>;

    // Add word to the global ignore list
    fn add_ignored(&mut self, word: &str) -> Result<i32>;
    // Add word to the ignore list for the given extension
    fn add_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()>;
    // Add word to the ignore list for the given file
    fn add_ignored_for_file(&mut self, word: &str, file: &str) -> Result<()>;

    fn lookup_word(&self, word: &str, file: &Path) -> Result<bool>;
}
