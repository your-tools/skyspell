/// Export a SystemDictionary that relies on Enchant Rust wrapper
use anyhow::{Context, Result, anyhow};

use crate::Dictionary;

pub struct SystemDictionary {
    dict: enchant::Dict,
    lang: String,
}

impl SystemDictionary {
    /// Must be called in main()
    pub fn init() {}

    pub fn new(lang: &str) -> Result<Self> {
        let mut broker = enchant::Broker::new();
        let dict_lang = Self::find_dict_lang(&mut broker, lang)
            .context(format!("No dict found for lang: '{lang}'"))?;
        let dict = broker
            .request_dict(&dict_lang)
            .map_err(|e| anyhow!("Could not request dict for lang '{lang}': {e}"))?;
        Ok(Self {
            dict,
            lang: lang.to_string(),
        })
    }

    fn find_dict_lang(broker: &mut enchant::Broker, lang: &str) -> Option<String> {
        let known_dicts = broker.list_dicts();
        let known_langs: Vec<_> = known_dicts.into_iter().map(|d| d.lang).collect();
        for known_lang in known_langs {
            if known_lang == lang {
                return Some(lang.to_string());
            }
            let before = known_lang
                .split('_')
                .next()
                .expect("split always yields on value");
            if before == lang {
                return Some(known_lang.to_string());
            }
        }

        None
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
