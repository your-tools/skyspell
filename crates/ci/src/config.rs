use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use skyspell_core::IgnoreStore;
use skyspell_core::ProjectId;
use skyspell_core::RelativePath;
use skyspell_core::Repository;

use crate::{CONFIG_FILE_NAME, PROJECT_ID};

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

#[derive(Serialize, Deserialize, Debug, Default)]
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

    pub fn init(lang: &str, provider: &str) -> Self {
        Self {
            lang: lang.to_string(),
            provider: provider.to_string(),
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
struct IgnoreConfig {
    #[serde(default)]
    global: Vec<String>,
    #[serde(default)]
    project: Vec<String>,
    #[serde(default)]
    extensions: BTreeMap<String, Vec<String>>,
    #[serde(default)]
    paths: BTreeMap<String, Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
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
        Ok(self.ignore.project.contains(&word))
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

// TODO: we have a ISP problem here
impl Repository for Config {
    fn insert_ignored_words(&mut self, _words: &[&str]) -> Result<()> {
        Ok(())
    }

    fn ignore(&mut self, word: &str) -> Result<()> {
        self.ignore.global.push(word.to_lowercase());
        Ok(())
    }

    fn new_project(&mut self, _project_path: &skyspell_core::ProjectPath) -> Result<ProjectId> {
        Ok(PROJECT_ID)
    }

    fn project_exists(&self, _project_path: &skyspell_core::ProjectPath) -> Result<bool> {
        Ok(true)
    }

    fn remove_project(&mut self, _project_id: ProjectId) -> Result<()> {
        Ok(())
    }

    fn get_project_id(&self, _project_path: &skyspell_core::ProjectPath) -> Result<ProjectId> {
        Ok(PROJECT_ID)
    }

    fn projects(&self) -> Result<Vec<skyspell_core::repository::ProjectInfo>> {
        Ok(vec![])
    }

    fn skip_file_name(&mut self, file_name: &str) -> Result<()> {
        self.skip.names.push(file_name.to_string());
        Ok(())
    }

    fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let entry = &mut self
            .ignore
            .extensions
            .entry(extension.to_string())
            .or_insert_with(Vec::new);
        entry.push(word.to_lowercase());
        Ok(())
    }

    fn ignore_for_project(&mut self, word: &str, _project_id: ProjectId) -> Result<()> {
        self.ignore(word)
    }

    fn ignore_for_path(
        &mut self,
        word: &str,
        _project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let entry = &mut self
            .ignore
            .paths
            .entry(relative_path.to_string())
            .or_insert_with(Vec::new);
        entry.push(word.to_lowercase());
        Ok(())
    }

    fn remove_ignored(&mut self, _word: &str) -> Result<()> {
        Ok(())
    }

    fn remove_ignored_for_extension(&mut self, _word: &str, _extension: &str) -> Result<()> {
        Ok(())
    }

    fn remove_ignored_for_path(
        &mut self,
        _word: &str,
        _project_id: ProjectId,
        _relative_path: &RelativePath,
    ) -> Result<()> {
        Ok(())
    }

    fn remove_ignored_for_project(&mut self, _word: &str, _project_id: ProjectId) -> Result<()> {
        Ok(())
    }

    fn skip_path(&mut self, _project_id: ProjectId, relative_path: &RelativePath) -> Result<()> {
        self.skip.paths.push(relative_path.to_string());
        Ok(())
    }

    fn unskip_file_name(&mut self, _file_name: &str) -> Result<()> {
        Ok(())
    }

    fn unskip_path(&mut self, _project_id: ProjectId, _relative_path: &RelativePath) -> Result<()> {
        Ok(())
    }

    fn insert_operation(
        &mut self,
        _operation: &skyspell_core::repository::Operation,
    ) -> Result<()> {
        Ok(())
    }

    fn pop_last_operation(&mut self) -> Result<Option<skyspell_core::repository::Operation>> {
        bail!("pop_last_operation is not implemented");
    }
}
