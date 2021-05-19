use anyhow::Result;

pub trait Repo {
    fn insert_good_words(&mut self, words: &[&str]) -> Result<()>;
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()>;

    fn known_extension(&self, ext: &str) -> Result<bool>;
    fn known_file(&self, path: &str) -> Result<bool>;

    fn add_extension(&mut self, ext: &str) -> Result<()>;
    fn add_file(&mut self, full_path: &str) -> Result<()>;

    fn add_ignored(&mut self, word: &str) -> Result<i32>;
    fn add_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()>;
    fn add_ignored_for_file(&mut self, word: &str, file: &str) -> Result<()>;

    fn lookup_word(&self, word: &str, file: Option<&str>, ext: Option<&str>) -> Result<bool>;
}
