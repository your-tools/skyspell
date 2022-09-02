use std::path::Path;

use anyhow::{bail, Result};

use crate::operations::Operation;
use crate::{IgnoreStore, Project, ProjectId, ProjectPath, RelativePath, Repository};

/// We have two backends to store ignore words One can manipulate ignored words,
/// but the other is more powerful because it can store and retrieve operations

/// Thus, we crate an enum to represent the "capabilities" of a storage - either
/// it implements Repository with all its methods, or it implements IgnoreStore
/// with a subset of these.
pub enum StorageBackend {
    IgnoreStore(Box<dyn IgnoreStore>),
    Repository(Box<dyn Repository>),
}

impl StorageBackend {
    pub fn ignore_store_mut(&mut self) -> &mut dyn IgnoreStore {
        match self {
            StorageBackend::IgnoreStore(i) => i.as_mut(),
            StorageBackend::Repository(r) => r.ignore_store_mut(),
        }
    }

    pub fn repository_mut(&mut self) -> Option<&mut dyn Repository> {
        match self {
            StorageBackend::IgnoreStore(_) => None,
            StorageBackend::Repository(r) => Some(r.as_mut()),
        }
    }

    pub fn ignore_store(&mut self) -> &mut dyn IgnoreStore {
        match self {
            StorageBackend::IgnoreStore(i) => i.as_mut(),
            StorageBackend::Repository(r) => r.ignore_store_mut(),
        }
    }

    pub(crate) fn should_ignore(
        &mut self,
        token: &str,
        project_id: i32,
        relative_path: &crate::RelativePath,
    ) -> Result<bool> {
        self.ignore_store()
            .should_ignore(token, project_id, relative_path)
    }

    pub fn is_ignored(&mut self, word: &str) -> Result<bool> {
        self.ignore_store().is_ignored(word)
    }

    pub fn is_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<bool> {
        self.ignore_store().is_ignored_for_extension(word, ext)
    }

    pub fn is_ignored_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<bool> {
        self.ignore_store().is_ignored_for_project(word, project_id)
    }

    pub fn is_ignored_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        self.ignore_store()
            .is_ignored_for_path(word, project_id, relative_path)
    }

    pub fn ignore(&mut self, word: &str) -> Result<()> {
        let operation = Operation::new_ignore(word);
        match self {
            StorageBackend::Repository(r) => {
                r.ignore_store_mut().ignore(word)?;
                r.insert_operation(&operation)
            }
            StorageBackend::IgnoreStore(i) => i.ignore(word),
        }
    }

    pub fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        let operation = Operation::new_ignore_for_project(word, project_id);
        match self {
            StorageBackend::Repository(r) => {
                r.ignore_store_mut().ignore_for_project(word, project_id)?;
                r.insert_operation(&operation)
            }
            StorageBackend::IgnoreStore(i) => i.ignore_for_project(word, project_id),
        }
    }

    pub fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let operation = Operation::new_ignore_for_path(word, project_id, relative_path);
        match self {
            StorageBackend::Repository(r) => {
                r.ignore_store_mut()
                    .ignore_for_path(word, project_id, relative_path)?;
                r.insert_operation(&operation)
            }
            StorageBackend::IgnoreStore(i) => i.ignore_for_path(word, project_id, relative_path),
        }
    }

    pub fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let operation = Operation::new_ignore_for_extension(word, extension);
        match self {
            StorageBackend::Repository(r) => {
                r.ignore_store_mut().ignore_for_extension(word, extension)?;
                r.insert_operation(&operation)
            }
            StorageBackend::IgnoreStore(i) => i.ignore_for_extension(word, extension),
        }
    }

    pub fn remove_ignored(&mut self, word: &str) -> Result<()> {
        self.ignore_store_mut().remove_ignored(word)
    }

    pub fn remove_ignored_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        self.ignore_store_mut()
            .remove_ignored_for_project(word, project_id)
    }

    pub fn remove_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()> {
        self.ignore_store_mut()
            .remove_ignored_for_extension(word, ext)
    }

    pub fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()> {
        self.ignore_store_mut()
            .remove_ignored_for_path(word, project_id, relative_path)
    }

    pub fn ensure_project(&mut self, project_path: &ProjectPath) -> Result<Project> {
        let project_id = match self.repository_mut() {
            Some(r) => {
                if !r.project_exists(project_path)? {
                    r.new_project(project_path)?;
                }
                r.get_project_id(project_path)?
            }
            None => 42,
        };

        Ok(Project::new(project_id, project_path.clone()))
    }

    pub fn new_project(&mut self, project_path: &ProjectPath) -> Result<Project> {
        let project_id = match self.repository_mut() {
            Some(r) => {
                r.new_project(project_path)?;
                r.get_project_id(project_path)?
            }
            None => 42,
        };

        Ok(Project::new(project_id, project_path.clone()))
    }

    pub fn clean(&mut self) -> Result<()> {
        let repository = match self {
            // No-op for IgnoreStore
            StorageBackend::IgnoreStore(_) => return Ok(()),
            StorageBackend::Repository(r) => r,
        };
        for project in repository.projects()? {
            let path = project.path();
            let path = Path::new(&path);
            let id = project.id();
            if !path.exists() {
                repository.remove_project(id)?;
                println!("Removed non longer existing project: {}", path.display());
            }
        }
        Ok(())
    }

    pub fn undo(&mut self) -> Result<()> {
        match self {
            StorageBackend::IgnoreStore(_) => bail!("Cannot undo"),
            StorageBackend::Repository(r) => r.undo(),
        }
    }
}
#[cfg(test)]
mod tests;
