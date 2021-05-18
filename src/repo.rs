use anyhow::Result;

#[derive(Debug)]
pub enum AddFor {
    NaturaLanguage,
    ProgrammingLanguage(i32),
    File(i32),
}

#[derive(Debug)]
pub enum Query<'a> {
    Simple(&'a str),
    ForProgrammingLanguage(&'a str, i32),
    ForFile(&'a str, i32),
    ForFileOrProgrammingLanguage(&'a str, i32, i32),
}

pub trait Repo {
    fn add_word(&self, word: &str, add_for: &AddFor) -> Result<()>;
    fn add_programming_language(&self, language: &str, extensions: &[&str]) -> Result<i32>;
    fn add_file(&self, path: &str) -> Result<i32>;
    fn lookup_extension(&self, ext: &str) -> Result<Option<i32>>;
    fn lookup_word(&self, query: &Query) -> Result<bool>;
}
