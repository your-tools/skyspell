use anyhow::{anyhow, bail, Context, Result};
use directories_next::BaseDirs;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};
use toml;

use crate::RelativePath;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GlobalIgnore {
    #[serde(default)]
    global: BTreeSet<String>,

    #[serde(default)]
    extensions: BTreeMap<String, BTreeSet<String>>,

    #[serde(default)]
    lang: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct LocalIgnore {
    #[serde(default)]
    pub patterns: BTreeSet<String>,

    #[serde(default)]
    project: BTreeSet<String>,

    #[serde(default)]
    paths: BTreeMap<String, BTreeSet<String>>,
}

impl LocalIgnore {
    pub fn load(path: &Path) -> Result<Self> {
        if path.exists() {
            load(path)
        } else {
            Ok(Default::default())
        }
    }
}

#[derive(Debug)]
pub struct IgnoreStore {
    global: GlobalIgnore,
    local: LocalIgnore,
    global_toml: PathBuf,
    local_toml: PathBuf,
}

fn load<T: DeserializeOwned + Default>(path: &Path) -> Result<T> {
    if !path.exists() {
        return Ok(Default::default());
    }
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("While reading {}:", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("While parsing {}:", path.display()))
}

fn save<T: Serialize>(name: &'static str, value: T, path: &Path) -> Result<()> {
    let contents = toml::ser::to_string_pretty(&value)
        .with_context(|| format!("while serializing {name} values"))?;
    std::fs::write(path, contents)
        .with_context(|| format!("while writing to {}", path.display()))?;
    Ok(())
}

pub fn global_path() -> Result<PathBuf> {
    let from_env = std::env::var("SKYSPELL_GLOBAL_PATH");
    if let Ok(value) = from_env {
        println!("Using {value} as global configuration file");
        return Ok(PathBuf::from(value));
    }

    let base_dirs = BaseDirs::new().ok_or_else(|| anyhow!("Could not get home directory"))?;
    let data_dir = base_dirs.data_dir().join("skyspell");
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Could not create data dir {}", data_dir.display()))?;
    Ok(data_dir.join("global.toml"))
}

/// Since the Win32 API and Enchant do not use the same language tags,
/// we remove only keep the stuff before `-` or `_` before storing them
/// in the global configuration file
fn short_lang(lang: &str) -> &str {
    lang.split(|c| c == '-' || c == '_')
        .next()
        .expect("calling next() after split() should always work")
}

impl IgnoreStore {
    pub fn load(global_toml: PathBuf, local_toml: PathBuf) -> Result<Self> {
        let global = load(&global_toml)?;
        let local = load(&local_toml)?;
        Ok(Self {
            global,
            local,
            global_toml,
            local_toml,
        })
    }

    // Should this word be ignored?
    // This is called when a word is *not* found in the spelling dictionary.
    //
    // A word is ignored if:
    //   * it's in the global ignore list
    //   * the relative path has an extension and it's in the ignore list
    //     for this extension
    //   * it's in the ignore list for the project
    //   * it's in the ignore list for the relative path
    //
    // Otherwise, it's *not* ignored and the Checker will call handle_error()
    //
    pub fn should_ignore(&self, word: &str, relative_path: &RelativePath, lang: &str) -> bool {
        if self.is_ignored(word) {
            return true;
        }

        if self.is_ignored_for_lang(word, lang) {
            return true;
        }

        if let Some(e) = relative_path.extension() {
            if self.is_ignored_for_extension(word, &e) {
                return true;
            }
        }

        if self.is_ignored_for_project(word) {
            return true;
        }

        if self.is_ignored_for_path(word, relative_path) {
            return true;
        }

        false
    }

    pub fn ignore(&mut self, word: &str) -> Result<()> {
        self.global.global.insert(word.to_owned());
        self.save_global()
    }

    pub fn is_ignored(&self, word: &str) -> bool {
        self.global.global.contains(word)
    }

    pub fn remove_ignored(&mut self, word: &str) -> Result<()> {
        let present = self.global.global.remove(word);
        if !present {
            bail!("word {word} was not ignored");
        }
        self.save_global()
    }

    pub fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let for_extension = self.global.extensions.get_mut(extension);
        match for_extension {
            Some(s) => {
                s.insert(word.to_owned());
            }
            None => {
                let mut set = BTreeSet::new();
                set.insert(word.to_owned());
                self.global.extensions.insert(extension.to_owned(), set);
            }
        };
        self.save_global()
    }

    pub fn is_ignored_for_extension(&self, word: &str, extension: &str) -> bool {
        let for_extension = self.global.extensions.get(extension);
        match for_extension {
            Some(s) => s.contains(word),
            None => false,
        }
    }

    pub fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        match self.global.extensions.get_mut(extension) {
            Some(set) => {
                set.remove(word);
            }
            None => bail!("{word} is not ignored for {extension}"),
        }
        self.save_global()
    }

    pub fn ignore_for_lang(&mut self, word: &str, lang: &str) -> Result<()> {
        let lang = short_lang(lang);
        let for_lang = self.global.lang.get_mut(lang);
        match for_lang {
            Some(s) => {
                s.insert(word.to_owned());
            }
            None => {
                let mut set = BTreeSet::new();
                set.insert(word.to_owned());
                self.global.lang.insert(lang.to_owned(), set);
            }
        };
        self.save_global()
    }

    pub fn is_ignored_for_lang(&self, word: &str, lang: &str) -> bool {
        let lang = short_lang(lang);
        let for_lang = self.global.lang.get(lang);
        match for_lang {
            Some(s) => s.contains(word),
            None => false,
        }
    }

    pub fn remove_ignored_for_lang(&mut self, word: &str, lang: &str) -> Result<()> {
        let lang = short_lang(lang);
        match self.global.lang.get_mut(lang) {
            Some(set) => {
                set.remove(word);
            }
            None => bail!("{word} is not ignored for {lang}"),
        }
        self.save_global()
    }

    pub fn ignore_for_project(&mut self, word: &str) -> Result<()> {
        self.local.project.insert(word.to_owned());
        self.save_local()
    }

    pub fn is_ignored_for_project(&self, word: &str) -> bool {
        self.local.project.contains(word)
    }

    pub fn remove_ignored_for_project(&mut self, word: &str) -> Result<()> {
        let present = self.local.project.remove(word);
        if !present {
            bail!("word {word} was not ignored");
        }
        self.save_local()
    }

    pub fn ignore_for_path(&mut self, word: &str, relative_path: &RelativePath) -> Result<()> {
        let path: &str = &relative_path.to_string();
        let for_path = self.local.paths.get_mut(path);
        match for_path {
            Some(s) => {
                s.insert(word.to_owned());
            }
            None => {
                let mut set = BTreeSet::new();
                set.insert(word.to_owned());
                self.local.paths.insert(path.to_owned(), set);
            }
        };
        self.save_local()
    }

    pub fn is_ignored_for_path(&self, word: &str, relative_path: &RelativePath) -> bool {
        let path: &str = &relative_path.to_string();
        let for_path = self.local.paths.get(path);
        match for_path {
            Some(s) => s.contains(word),
            None => false,
        }
    }

    pub fn remove_ignored_for_path(
        &mut self,
        word: &str,
        relative_path: &crate::RelativePath,
    ) -> Result<()> {
        let path: &str = &relative_path.to_string();
        match self.local.paths.get_mut(path) {
            Some(set) => {
                set.remove(word);
            }
            None => bail!("{word} is not ignored path {path}"),
        }
        self.save_local()
    }

    fn save_global(&self) -> Result<()> {
        save("global", &self.global, &self.global_toml)
    }

    fn save_local(&self) -> Result<()> {
        save("local", &self.local, &self.local_toml)
    }
}

#[cfg(test)]
mod tests;
