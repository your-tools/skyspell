use anyhow::Result;

pub trait Dictionary {
    // Check if the word is found in the dictionary
    fn check(&self, word: &str) -> Result<bool>;
    // Suggest replacement for error string
    fn suggest(&self, error: &str) -> Result<Vec<String>>;
    fn lang(&self) -> &str;
    fn provider(&self) -> &str;
}
