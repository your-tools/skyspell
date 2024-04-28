#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::Config;
use crate::RelativePath;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum Operation {
    Ignore(Ignore),
    IgnoreForExtension(IgnoreForExtension),
    IgnoreForPath(IgnoreForPath),
    IgnoreForProject(IgnoreForProject),
}

// Note: this is a bit verbose but less than coming up with a trait
// that must be implemented for each variant
impl Operation {
    pub fn new_ignore(word: &str) -> Self {
        Self::Ignore(Ignore {
            word: word.to_string(),
        })
    }
    pub fn new_ignore_for_project(word: &str) -> Self {
        Self::IgnoreForProject(IgnoreForProject {
            word: word.to_string(),
        })
    }

    pub fn new_ignore_for_path(word: &str, relative_path: &RelativePath) -> Self {
        Self::IgnoreForPath(IgnoreForPath {
            word: word.to_string(),
            path: relative_path.clone(),
        })
    }

    pub fn new_ignore_for_extension(word: &str, extension: &str) -> Self {
        Self::IgnoreForExtension(IgnoreForExtension {
            word: word.to_string(),
            extension: extension.to_string(),
        })
    }

    pub fn execute(&mut self, config: &mut Config) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.execute(config),
            IgnoreForExtension(o) => o.execute(config),
            IgnoreForPath(o) => o.execute(config),
            IgnoreForProject(o) => o.execute(config),
        }
    }

    pub fn undo(&mut self, config: &mut Config) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.undo(config),
            IgnoreForExtension(o) => o.undo(config),
            IgnoreForPath(o) => o.undo(config),
            IgnoreForProject(o) => o.undo(config),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Ignore {
    pub word: String,
}

impl Ignore {
    fn execute(&mut self, config: &mut Config) -> Result<()> {
        config.ignore(&self.word)
    }

    fn undo(&mut self, config: &mut Config) -> Result<()> {
        config.remove_ignored(&self.word)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct IgnoreForExtension {
    word: String,
    extension: String,
}

impl IgnoreForExtension {
    fn execute(&mut self, config: &mut Config) -> Result<()> {
        config.ignore_for_extension(&self.word, &self.extension)
    }

    fn undo(&mut self, config: &mut Config) -> Result<()> {
        config.remove_ignored_for_extension(&self.word, &self.extension)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct IgnoreForProject {
    word: String,
}

impl IgnoreForProject {
    fn execute(&mut self, config: &mut Config) -> Result<()> {
        config.ignore_for_project(&self.word)
    }

    fn undo(&mut self, config: &mut Config) -> Result<()> {
        config.remove_ignored_for_project(&self.word)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct IgnoreForPath {
    word: String,
    path: RelativePath,
}

impl IgnoreForPath {
    fn execute(&mut self, config: &mut Config) -> Result<()> {
        config.ignore_for_path(&self.word, &self.path)
    }

    fn undo(&mut self, config: &mut Config) -> Result<()> {
        config.remove_ignored_for_path(&self.word, &self.path)
    }
}
