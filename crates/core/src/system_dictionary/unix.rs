/// Export a SystemDictionary that relies on Enchant Rust wrapper
use anyhow::{anyhow, Result};

use crate::Dictionary;

pub struct SystemDictionary {
    dict: enchant::Dict,
    lang: String,
}

impl SystemDictionary {
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

impl Dictionary for SystemDictionary {
    fn check(&self, word: &str) -> Result<bool> {
        self.dict
            .check(word)
            .map_err(|e| anyhow!("Could not check '{word}' with enchant: {e}"))
    }

    fn suggest(&self, error: &str) -> Result<Vec<String>> {
        Ok(self.dict.suggest(error))
    }

    fn lang(&self) -> &str {
        &self.lang
    }

    fn provider(&self) -> &str {
        self.dict.get_provider_name()
    }
}
