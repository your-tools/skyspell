#![allow(dead_code)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::IgnoreStore;
use crate::ProjectId;
use crate::RelativePath;

pub struct Undoer<I: IgnoreStore> {
    ignore_store: I,
}

impl<I: IgnoreStore> Undoer<I> {
    pub fn new(ignore_store: I) -> Self {
        Self { ignore_store }
    }

    pub fn ignore_store(&self) -> &dyn IgnoreStore {
        &self.ignore_store
    }

    pub fn ignore_store_mut(&mut self) -> &mut I {
        &mut self.ignore_store
    }

    fn run(&mut self, mut operation: Operation) -> Result<()> {
        operation.execute(&mut self.ignore_store)?;
        todo!()
    }

    pub fn undo(&mut self) -> Result<()> {
        todo!()
    }

    pub fn ignore(&mut self, word: &str) -> Result<()> {
        self.run(Operation::Ignore(Ignore {
            word: word.to_string(),
        }))
    }

    pub fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        self.run(Operation::IgnoreForExtension(IgnoreForExtension {
            word: word.to_string(),
            extension: extension.to_string(),
        }))
    }

    pub fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        self.run(Operation::IgnoreForProject(IgnoreForProject {
            word: word.to_string(),
            project_id,
        }))
    }

    pub fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        path: &RelativePath,
    ) -> Result<()> {
        self.run(Operation::IgnoreForPath(IgnoreForPath {
            word: word.to_string(),
            project_id,
            path: path.clone(),
        }))
    }
}

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
    fn execute<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.execute(repo),
            IgnoreForExtension(o) => o.execute(repo),
            IgnoreForPath(o) => o.execute(repo),
            IgnoreForProject(o) => o.execute(repo),
        }
    }

    fn undo<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.undo(repo),
            IgnoreForExtension(o) => o.undo(repo),
            IgnoreForPath(o) => o.undo(repo),
            IgnoreForProject(o) => o.undo(repo),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ignore {
    pub word: String,
}

impl Ignore {
    fn execute<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        repo.ignore(&self.word)
    }

    fn undo<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        repo.remove_ignored(&self.word)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IgnoreForExtension {
    word: String,
    extension: String,
}

impl IgnoreForExtension {
    fn execute<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        repo.ignore_for_extension(&self.word, &self.extension)
    }

    fn undo<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        repo.remove_ignored_for_extension(&self.word, &self.extension)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IgnoreForProject {
    word: String,
    project_id: ProjectId,
}

impl IgnoreForProject {
    fn execute<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        repo.ignore_for_project(&self.word, self.project_id)
    }

    fn undo<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        repo.remove_ignored_for_project(&self.word, self.project_id)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IgnoreForPath {
    word: String,
    project_id: ProjectId,
    path: RelativePath,
}

impl IgnoreForPath {
    fn execute<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        repo.ignore_for_path(&self.word, self.project_id, &self.path)
    }

    fn undo<I: IgnoreStore>(&mut self, repo: &mut I) -> Result<()> {
        repo.remove_ignored_for_path(&self.word, self.project_id, &self.path)
    }
}
