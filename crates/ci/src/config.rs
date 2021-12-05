use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use skyspell_core::IgnoreStore;
use skyspell_core::ProjectId;
use skyspell_core::RelativePath;

use crate::CONFIG_FILE_NAME;

pub fn parse_config(config_path: &Path) -> Result<Config> {
    let config_text = std::fs::read_to_string(config_path)
        .with_context(|| format!("Error when reading {:?}", config_path))?;
    let config: Config = serde_yaml::from_str(&config_text)
        .with_context(|| format!("Error when parsing {:?}", config_path))?;

    let errors = validate_config(&config);
    if errors.is_empty() {
        return Ok(config);
    }

    for error in errors {
        eprintln!("{}", error);
    }

    bail!("Invalid config");
}

fn validate_config(config: &Config) -> Vec<String> {
    let mut errors = vec![];
    for ignore_path in config.ignore.paths.keys() {
        let path = Path::new(&ignore_path);
        if !path.exists() {
            errors.push(format!("Ignored path: '{}' does not exist", ignore_path));
        }
    }
    errors
}

#[derive(Deserialize, Debug)]
pub struct Config {
    lang: String,
    provider: String,
    #[serde(default)]
    ignore: IgnoreConfig,
    #[serde(default)]
    skip: SkipConfig,
}

impl Config {
    pub fn lang(&self) -> &str {
        &self.lang
    }

    pub fn provider(&self) -> &str {
        &self.provider
    }
}

#[derive(Deserialize, Debug, Default)]
struct IgnoreConfig {
    #[serde(default)]
    global: Vec<String>,
    #[serde(default)]
    extensions: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    paths: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize, Debug, Default)]
struct SkipConfig {
    #[serde(default)]
    paths: Vec<String>,
    #[serde(default)]
    names: Vec<String>,
}

impl IgnoreStore for Config {
    fn is_ignored(&self, word: &str) -> Result<bool> {
        let word = word.to_lowercase();
        Ok(self.ignore.global.contains(&word))
    }

    fn is_skipped_file_name(&self, file_name: &str) -> Result<bool> {
        Ok(self.skip.names.contains(&file_name.to_string()))
    }

    fn is_ignored_for_extension(&self, word: &str, extension: &str) -> Result<bool> {
        let word = word.to_lowercase();
        if let Some(words) = self.ignore.extensions.get(extension) {
            Ok(words.contains(&word))
        } else {
            Ok(false)
        }
    }

    fn is_ignored_for_project(&self, word: &str, _project_id: ProjectId) -> Result<bool> {
        let word = word.to_lowercase();
        self.is_ignored(&word)
    }

    fn is_ignored_for_path(
        &self,
        word: &str,
        _project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        let word = word.to_lowercase();
        if let Some(words) = self.ignore.paths.get(&relative_path.to_string()) {
            Ok(words.contains(&word))
        } else {
            Ok(false)
        }
    }

    fn is_skipped_path(
        &self,
        _project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        let as_string = relative_path.to_string();
        // Always skip our config file
        if as_string == CONFIG_FILE_NAME {
            return Ok(true);
        }
        Ok(self.skip.paths.contains(&relative_path.to_string()))
    }
}
