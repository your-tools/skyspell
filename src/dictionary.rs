use anyhow::{anyhow, Result};

pub trait Dictionary {
    // Check if the word is found in the dictionary
    fn check(&self, word: &str) -> Result<bool>;
    // Suggest replacement for error string
    fn suggest(&self, error: &str) -> Vec<String>;
    fn lang(&self) -> &str;
}

pub struct EnchantDictionary {
    dict: enchant::Dict,
    lang: String,
}

impl EnchantDictionary {
    pub fn new(broker: &mut enchant::Broker, lang: &str) -> Result<Self> {
        let dict = broker
            .request_dict(lang)
            .map_err(|e| anyhow!("Could not request dict for lang '{}': {}", lang, e))?;
        Ok(Self {
            dict,
            lang: lang.to_string(),
        })
    }
}

impl Dictionary for EnchantDictionary {
    fn check(&self, word: &str) -> Result<bool> {
        self.dict
            .check(word)
            .map_err(|e| anyhow!("Could not check '{}' with enchant: {}", word, e))
    }

    fn suggest(&self, error: &str) -> Vec<String> {
        self.dict.suggest(error)
    }

    fn lang(&self) -> &str {
        &self.lang
    }
}

#[cfg(test)]
mod tests;
