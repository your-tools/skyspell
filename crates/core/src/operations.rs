#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::IgnoreConfig;
use crate::RelativePath;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Operation {
    Ignore(Ignore),
    IgnoreForExtension(IgnoreForExtension),
    IgnoreForPath(IgnoreForPath),
    IgnoreForProject(IgnoreForProject),
}

// Note: this is a bit verbose but less than coming up with a trait
// that must be implemented for each variant
impl Operation {
    pub(crate) fn new_ignore(word: &str) -> Self {
        Self::Ignore(Ignore {
            word: word.to_string(),
        })
    }
    pub(crate) fn new_ignore_for_project(word: &str) -> Self {
        Self::IgnoreForProject(IgnoreForProject {
            word: word.to_string(),
        })
    }

    pub(crate) fn new_ignore_for_path(word: &str, relative_path: &RelativePath) -> Self {
        Self::IgnoreForPath(IgnoreForPath {
            word: word.to_string(),
            path: relative_path.clone(),
        })
    }

    pub(crate) fn new_ignore_for_extension(word: &str, extension: &str) -> Self {
        Self::IgnoreForExtension(IgnoreForExtension {
            word: word.to_string(),
            extension: extension.to_string(),
        })
    }

    pub fn execute(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.execute(ignore_store),
            IgnoreForExtension(o) => o.execute(ignore_store),
            IgnoreForPath(o) => o.execute(ignore_store),
            IgnoreForProject(o) => o.execute(ignore_store),
        }
    }

    pub fn undo(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.undo(ignore_store),
            IgnoreForExtension(o) => o.undo(ignore_store),
            IgnoreForPath(o) => o.undo(ignore_store),
            IgnoreForProject(o) => o.undo(ignore_store),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ignore {
    pub word: String,
}

impl Ignore {
    fn execute(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        ignore_store.ignore(&self.word)
    }

    fn undo(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        ignore_store.remove_ignored(&self.word)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IgnoreForExtension {
    word: String,
    extension: String,
}

impl IgnoreForExtension {
    fn execute(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        ignore_store.ignore_for_extension(&self.word, &self.extension)
    }

    fn undo(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        ignore_store.remove_ignored_for_extension(&self.word, &self.extension)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IgnoreForProject {
    word: String,
}

impl IgnoreForProject {
    fn execute(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        ignore_store.ignore_for_project(&self.word)
    }

    fn undo(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        ignore_store.remove_ignored_for_project(&self.word)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IgnoreForPath {
    word: String,
    path: RelativePath,
}

impl IgnoreForPath {
    fn execute(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        ignore_store.ignore_for_path(&self.word, &self.path)
    }

    fn undo(&mut self, ignore_store: &mut IgnoreConfig) -> Result<()> {
        ignore_store.remove_ignored_for_path(&self.word, &self.path)
    }
}
