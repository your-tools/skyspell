use crate::RelativePath;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Serialize, Deserialize, Debug, Default)]
struct InnerConfig {
    #[serde(default)]
    provider: Option<String>,

    #[serde(default)]
    use_db: bool,

    #[serde(default)]
    ignore: Ignore,
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct Ignore {
    #[serde(default)]
    patterns: BTreeSet<String>,

    #[serde(default)]
    global: BTreeSet<String>,

    #[serde(default)]
    project: BTreeSet<String>,

    #[serde(default)]
    extensions: BTreeMap<String, BTreeSet<String>>,

    #[serde(default)]
    paths: BTreeMap<String, BTreeSet<String>>,
}

#[derive(Debug, Default)]
pub struct Config {
    inner: InnerConfig,
    path: Option<PathBuf>,
}

impl Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl Config {
    pub fn open_or_create(config_path: &Path) -> Result<Self> {
        if !config_path.exists() {
            std::fs::write(config_path, "").with_context(|| {
                format!("While creating empty config at {}:", config_path.display())
            })?;
        }
        let contents = std::fs::read_to_string(config_path)
            .with_context(|| format!("While reading {}:", config_path.display()))?;
        let config: InnerConfig = toml_edit::de::from_str(&contents)
            .with_context(|| format!("While parsing {}:", config_path.display()))?;
        Ok(Config {
            path: Some(config_path.to_path_buf()),
            inner: config,
        })
    }

    pub fn empty() -> Self {
        Self {
            path: None,
            inner: Default::default(),
        }
    }

    pub fn parse(contents: &str) -> Result<Self> {
        let ignore: InnerConfig = toml_edit::de::from_str(contents)?;
        Ok(Config {
            inner: ignore,
            path: None,
        })
    }

    pub fn provider(&self) -> &Option<String> {
        &self.inner.provider
    }

    pub fn patterns(&self) -> Vec<&String> {
        self.inner.ignore.patterns.iter().collect::<Vec<_>>()
    }

    pub fn use_db(&self) -> bool {
        self.inner.use_db
    }

    fn save(&self) -> Result<()> {
        let path = match &self.path {
            None => return Ok(()),
            Some(p) => p,
        };
        let contents = toml_edit::ser::to_string_pretty(&self.inner)
            .with_context(|| "Could not serialize config")?;
        std::fs::write(path, contents)
            .with_context(|| format!("Could not save config in {}", path.display()))?;
        Ok(())
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
    pub fn should_ignore(&mut self, word: &str, relative_path: &RelativePath) -> Result<bool> {
        if self.is_ignored(word)? {
            return Ok(true);
        }

        if let Some(e) = relative_path.extension() {
            if self.is_ignored_for_extension(word, &e)? {
                return Ok(true);
            }
        }

        if self.is_ignored_for_project(word)? {
            return Ok(true);
        }

        self.is_ignored_for_path(word, relative_path)
    }

    pub fn is_ignored(&mut self, word: &str) -> Result<bool> {
        Ok(self.inner.ignore.global.contains(word))
    }

    pub fn is_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<bool> {
        let for_extension = self.inner.ignore.extensions.get(extension);
        Ok(match for_extension {
            Some(s) => s.contains(word),
            None => false,
        })
    }

    pub fn is_ignored_for_project(&mut self, word: &str) -> Result<bool> {
        Ok(self.inner.ignore.project.contains(word))
    }

    pub fn is_ignored_for_path(
        &mut self,
        word: &str,
        relative_path: &crate::RelativePath,
    ) -> Result<bool> {
        let path: &str = &relative_path.as_str();
        let for_path = self.inner.ignore.paths.get(&path.to_owned());
        Ok(match for_path {
            Some(s) => s.contains(word),
            None => false,
        })
    }

    pub fn ignore(&mut self, word: &str) -> Result<()> {
        self.inner.ignore.global.insert(word.to_owned());
        self.save()
    }

    pub fn ignore_for_extension(&mut self, word: &str, ext: &str) -> Result<()> {
        let for_extension = self.inner.ignore.extensions.get_mut(ext);
        match for_extension {
            Some(s) => {
                s.insert(word.to_owned());
            }
            None => {
                let mut set = BTreeSet::new();
                set.insert(word.to_owned());
                self.inner.ignore.extensions.insert(ext.to_owned(), set);
            }
        };
        self.save()
    }

    pub fn ignore_for_project(&mut self, word: &str) -> Result<()> {
        self.inner.ignore.project.insert(word.to_owned());
        self.save()
    }

    pub fn ignore_for_path(
        &mut self,
        word: &str,
        relative_path: &crate::RelativePath,
    ) -> Result<()> {
        let path: &str = &relative_path.as_str();
        let for_path = self.inner.ignore.paths.get_mut(path);
        match for_path {
            Some(s) => {
                s.insert(word.to_owned());
            }
            None => {
                let mut set = BTreeSet::new();
                set.insert(word.to_owned());
                self.inner.ignore.paths.insert(path.to_owned(), set);
            }
        };
        self.save()
    }

    pub fn remove_ignored(&mut self, word: &str) -> Result<()> {
        let present = self.inner.ignore.global.remove(word);
        if !present {
            bail!("word {word} was not ignored");
        }
        self.save()
    }

    pub fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        match self.inner.ignore.extensions.get_mut(extension) {
            Some(set) => {
                set.remove(word);
            }
            None => bail!("{word} is not ignored for {extension}"),
        }
        self.save()
    }

    pub fn remove_ignored_for_path(
        &mut self,
        word: &str,
        relative_path: &crate::RelativePath,
    ) -> Result<()> {
        let path: &str = &relative_path.as_str();
        match self.inner.ignore.paths.get_mut(path) {
            Some(set) => {
                set.remove(word);
            }
            None => bail!("{word} is not ignored path {path}"),
        }
        self.save()
    }

    pub fn remove_ignored_for_project(&mut self, word: &str) -> Result<()> {
        let present = self.inner.ignore.project.remove(word);
        if !present {
            bail!("word {word} was not ignored");
        }
        self.save()
    }
}

#[cfg(test)]
mod tests;
