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
            .map_err(|e| anyhow!("Could not request dict for lang {}: {}", lang, e))?;
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
mod tests {
    // Note: these tests will fail if for some reason 'missstake' is in
    // the personal dict, or if no Enchant provider for the US English dictionary is found,
    // and there's no good way to know :(

    use super::*;

    #[test]
    fn test_check() {
        let mut broker = enchant::Broker::new();
        let dict = EnchantDictionary::new(&mut broker, "en_US").unwrap();
        assert!(!dict.check("missstake").unwrap());
    }

    #[test]
    fn test_suggest() {
        let mut broker = enchant::Broker::new();
        let dict = EnchantDictionary::new(&mut broker, "en_US").unwrap();
        assert!(!dict.check("missstake").unwrap());
    }
}
