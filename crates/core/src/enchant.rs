use anyhow::{anyhow, Result};

use crate::Dictionary;

pub struct EnchantDictionary {
    dict: enchant::Dict,
    lang: String,
}

impl EnchantDictionary {
    pub fn new(lang: &str) -> Result<Self> {
        let mut broker = enchant::Broker::new();
        let dict = broker
            .request_dict(lang)
            .map_err(|e| anyhow!("Could not request dict for lang '{lang}': {e}"))?;
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
            .map_err(|e| anyhow!("Could not check '{word}' with enchant: {e}"))
    }

    fn suggest(&self, error: &str) -> Vec<String> {
        self.dict.suggest(error)
    }

    fn lang(&self) -> &str {
        &self.lang
    }

    fn provider(&self) -> &str {
        self.dict.get_provider_name()
    }
}

#[cfg(test)]
mod tests;
