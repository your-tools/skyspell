use anyhow::Result;

mod wrapper;

pub struct AspellDictionary {
    speller: wrapper::Speller,
    lang: String,
}

impl AspellDictionary {
    pub fn new(lang: &str) -> Result<Self> {
        let mut config = wrapper::Config::new();
        config.set_lang(lang)?;
        // We don't want our CI to use the "personal" dictionaries that
        // may be present in the home directory (like  ~/.aspell.en.pws)
        config.use_other_dicts(false)?;
        let speller = config.speller()?;
        Ok(Self {
            speller,
            lang: lang.to_string(),
        })
    }
}

impl crate::Dictionary for AspellDictionary {
    fn check(&self, word: &str) -> Result<bool> {
        self.speller.check(word)
    }

    fn suggest(&self, error: &str) -> Vec<String> {
        self.speller.suggest(error)
    }

    fn lang(&self) -> &str {
        &self.lang
    }

    fn provider(&self) -> &str {
        "aspell"
    }
}

#[cfg(test)]
mod tests;
