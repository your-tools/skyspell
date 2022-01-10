use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

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

    pub fn ignored(&self) -> impl Iterator<Item = &str> {
        self.ignore.global.iter().map(|x| x.as_ref())
    }

    pub fn ignored_for_project(&self) -> impl Iterator<Item = &str> {
        self.ignore.project.iter().map(|x| x.as_ref())
    }

    pub fn extensions(&self) -> impl Iterator<Item = &str> {
        self.ignore.extensions.keys().map(|x| x.as_ref())
    }

    // Note: you should call this on an extension returned by self::extensions(),
    // otherwise the code will panic
    pub fn by_extension(&self, extension: &str) -> impl Iterator<Item = &str> {
        let values = &self.ignore.extensions[extension];
        values.iter().map(|x| x.as_ref())
    }

    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.ignore.paths.keys().map(|x| x.as_ref())
    }

    // Note: you should call this on an extension returned by self::extensions(),
    // otherwise the code will panic
    pub fn by_path(&self, path: &str) -> impl Iterator<Item = &str> {
        let values = &self.ignore.paths[path];
        values.iter().map(|x| x.as_ref())
    }

    pub fn skipped_file_names(&self) -> &[String] {
        &self.skip.names
    }

    pub fn skipped_paths(&self) -> &[String] {
        &self.skip.paths
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
