use std::sync::Once;

use anyhow::{bail, Result};
use windows::{
    core::{HSTRING, PWSTR},
    Win32::{
        Globalization::{ISpellChecker, ISpellCheckerFactory, SpellCheckerFactory},
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED,
        },
    },
};

struct SpellClient {
    spell_checker: ISpellChecker,
}

static INIT: Once = Once::new();

pub fn initialize_com() {
    INIT.call_once(|| unsafe {
        CoInitializeEx(None, COINIT_MULTITHREADED).unwrap();
    });
}

impl SpellClient {
    fn new(lang: &str) -> Result<Self> {
        initialize_com();
        let spell_checker = unsafe {
            let language_tag = HSTRING::from(lang);
            let spell_checker_factory: ISpellCheckerFactory =
                CoCreateInstance(&SpellCheckerFactory, None, CLSCTX_ALL)?;
            let is_supported = spell_checker_factory.IsSupported(&language_tag)?.as_bool();
            if !is_supported {
                bail!("Language '{lang}' is not supported")
            }

            spell_checker_factory.CreateSpellChecker(&language_tag)?
        };
        Ok(Self { spell_checker })
    }

    fn check(&self, word: &str) -> anyhow::Result<bool> {
        let error = unsafe {
            let text = HSTRING::from(word);
            let spelling_errors = self.spell_checker.Check(&text)?;
            let mut spelling_error = None;
            let result = spelling_errors.Next(&mut spelling_error);
            if result.is_err() {
                bail!("When getting next error: {}", result.message());
            }
            spelling_error
        };
        Ok(error.is_none())
    }

    fn suggest(&self, word: &str) -> anyhow::Result<Vec<String>> {
        let word = HSTRING::from(word);
        let mut suggestions = vec![];
        unsafe {
            let enum_string = self.spell_checker.Suggest(&word)?;

            loop {
                // Get the next suggestion breaking if the call to `Next` failed
                let mut wstring_pointers = [PWSTR::null()];
                _ = enum_string.Next(&mut wstring_pointers, None);
                if wstring_pointers[0].is_null() {
                    break;
                }

                let as_string = wstring_pointers[0].to_string()?;
                suggestions.push(as_string);

                CoTaskMemFree(Some(wstring_pointers[0].as_ptr() as *mut _));
            }
        }
        Ok(suggestions)
    }
}

#[cfg(test)]
mod tests;
