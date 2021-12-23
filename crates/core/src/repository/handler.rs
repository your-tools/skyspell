#![allow(dead_code)]

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::IgnoreStore;
use crate::ProjectId;
use crate::RelativePath;
use crate::Repository;

pub struct RepositoryHandler<R: Repository> {
    repository: R,
}

impl<R: Repository> RepositoryHandler<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub fn as_ignore_store(&self) -> &dyn IgnoreStore {
        &self.repository
    }

    pub fn repository(&mut self) -> &mut R {
        &mut self.repository
    }

    fn run(&mut self, mut operation: Operation) -> Result<()> {
        operation.execute(&mut self.repository)?;
        self.repository.insert_operation(&operation)
    }

    pub fn undo(&mut self) -> Result<()> {
        let last_operation = self.repository.pop_last_operation()?;
        let mut last_operation = last_operation.ok_or_else(|| anyhow!("Nothing to undo"))?;
        last_operation.undo(&mut self.repository)
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

    pub fn skip_file_name(&mut self, file_name: &str) -> Result<()> {
        self.run(Operation::SkipFileName(SkipFileName {
            file_name: file_name.to_string(),
        }))
    }

    pub fn skip_path(&mut self, project_id: ProjectId, path: &RelativePath) -> Result<()> {
        self.run(Operation::SkipPath(SkipPath {
            project_id,
            path: path.clone(),
        }))
    }

    // Note: used for core tests
    #[allow(dead_code)]
    pub fn is_skipped_file_name(&mut self, file_name: &str) -> Result<bool> {
        self.repository.is_skipped_file_name(file_name)
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Operation {
    Ignore(Ignore),
    IgnoreForExtension(IgnoreForExtension),
    IgnoreForPath(IgnoreForPath),
    IgnoreForProject(IgnoreForProject),
    SkipFileName(SkipFileName),
    SkipPath(SkipPath),
}

// Note: this is a bit verbose but less than coming up with a trait
// that must be implemented for each variant
impl Operation {
    fn execute<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.execute(repo),
            IgnoreForExtension(o) => o.execute(repo),
            IgnoreForPath(o) => o.execute(repo),
            IgnoreForProject(o) => o.execute(repo),
            SkipFileName(o) => o.execute(repo),
            SkipPath(o) => o.execute(repo),
        }
    }

    fn undo<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        use Operation::*;
        match self {
            Ignore(o) => o.undo(repo),
            IgnoreForExtension(o) => o.undo(repo),
            IgnoreForPath(o) => o.undo(repo),
            IgnoreForProject(o) => o.undo(repo),
            SkipFileName(o) => o.undo(repo),
            SkipPath(o) => o.undo(repo),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ignore {
    pub word: String,
}

impl Ignore {
    fn execute<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.ignore(&self.word)
    }

    fn undo<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.remove_ignored(&self.word)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IgnoreForExtension {
    word: String,
    extension: String,
}

impl IgnoreForExtension {
    fn execute<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.ignore_for_extension(&self.word, &self.extension)
    }

    fn undo<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.remove_ignored_for_extension(&self.word, &self.extension)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct IgnoreForProject {
    word: String,
    project_id: ProjectId,
}

impl IgnoreForProject {
    fn execute<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.ignore_for_project(&self.word, self.project_id)
    }

    fn undo<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
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
    fn execute<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.ignore_for_path(&self.word, self.project_id, &self.path)
    }

    fn undo<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.remove_ignored_for_path(&self.word, self.project_id, &self.path)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkipFileName {
    file_name: String,
}

impl SkipFileName {
    fn execute<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.skip_file_name(&self.file_name)
    }

    fn undo<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.unskip_file_name(&self.file_name)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SkipPath {
    project_id: ProjectId,
    path: RelativePath,
}

impl SkipPath {
    fn execute<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.skip_path(self.project_id, &self.path)
    }

    fn undo<R: Repository>(&mut self, repo: &mut R) -> Result<()> {
        repo.unskip_path(self.project_id, &self.path)
    }
}
