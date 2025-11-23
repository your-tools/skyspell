#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::IgnoreStore;
use crate::ProjectFile;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone)]
pub enum Operation {
    Ignore(Ignore),
    IgnoreForExtension(IgnoreForExtension),
    IgnoreForPath(IgnoreForPath),
    IgnoreForProject(IgnoreForProject),
    IgnoreForLang(IgnoreForLang),
}

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

    pub fn new_ignore_for_path(word: &str, project_file: &ProjectFile) -> Self {
        Self::IgnoreForPath(IgnoreForPath {
            word: word.to_string(),
            project_file: project_file.clone(),
        })
    }

    pub fn new_ignore_for_extension(word: &str, extension: &str) -> Self {
        Self::IgnoreForExtension(IgnoreForExtension {
            word: word.to_string(),
            extension: extension.to_string(),
        })
    }

    pub fn new_ignore_for_lang(word: &str, lang: &str) -> Self {
        Self::IgnoreForLang(IgnoreForLang {
            word: word.to_string(),
            lang: lang.to_string(),
        })
    }

    pub fn execute(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.execute(ignore_store),
            IgnoreForExtension(o) => o.execute(ignore_store),
            IgnoreForLang(o) => o.execute(ignore_store),
            IgnoreForPath(o) => o.execute(ignore_store),
            IgnoreForProject(o) => o.execute(ignore_store),
        }
    }

    pub fn undo(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.undo(ignore_store),
            IgnoreForExtension(o) => o.undo(ignore_store),
            IgnoreForLang(o) => o.undo(ignore_store),
            IgnoreForPath(o) => o.undo(ignore_store),
            IgnoreForProject(o) => o.undo(ignore_store),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Ignore {
    pub word: String,
}

impl Ignore {
    fn execute(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.ignore(&self.word)
    }

    fn undo(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.remove_ignored(&self.word)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct IgnoreForExtension {
    word: String,
    extension: String,
}

impl IgnoreForExtension {
    fn execute(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.ignore_for_extension(&self.word, &self.extension)
    }

    fn undo(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.remove_ignored_for_extension(&self.word, &self.extension)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct IgnoreForLang {
    word: String,
    lang: String,
}

impl IgnoreForLang {
    fn execute(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.ignore_for_lang(&self.word, &self.lang)
    }

    fn undo(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.remove_ignored_for_lang(&self.word, &self.lang)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct IgnoreForProject {
    word: String,
}

impl IgnoreForProject {
    fn execute(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.ignore_for_project(&self.word)
    }

    fn undo(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.remove_ignored_for_project(&self.word)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct IgnoreForPath {
    word: String,
    project_file: ProjectFile,
}

impl IgnoreForPath {
    fn execute(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.ignore_for_path(&self.word, &self.project_file)
    }

    fn undo(&mut self, ignore_store: &mut IgnoreStore) -> Result<()> {
        ignore_store.remove_ignored_for_path(&self.word, &self.project_file)
    }
}

#[cfg(test)]
mod tests;
