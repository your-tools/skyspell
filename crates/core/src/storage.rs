use anyhow::{bail, Result};

use crate::operations::Operation;
use crate::{IgnoreStore, Project, ProjectId, ProjectPath, RelativePath};

pub enum StorageBackend {
    IgnoreStore(Box<dyn IgnoreStore>),
}

impl StorageBackend {
    pub fn ignore_store_mut(&mut self) -> &mut dyn IgnoreStore {
        match self {
            StorageBackend::IgnoreStore(i) => i.as_mut(),
        }
    }

    pub fn ignore_store(&mut self) -> &mut dyn IgnoreStore {
        match self {
            StorageBackend::IgnoreStore(i) => i.as_mut(),
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
        let _operation = Operation::new_ignore(word);
        match self {
            StorageBackend::IgnoreStore(i) => i.ignore(word),
        }
    }

    pub fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        let _operation = Operation::new_ignore_for_project(word, project_id);
        match self {
            StorageBackend::IgnoreStore(i) => i.ignore_for_project(word, project_id),
        }
    }

    pub fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let _operation = Operation::new_ignore_for_path(word, project_id, relative_path);
        match self {
            StorageBackend::IgnoreStore(i) => i.ignore_for_path(word, project_id, relative_path),
        }
    }

    pub fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let _operation = Operation::new_ignore_for_extension(word, extension);
        match self {
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
        Ok(Project::new(42, project_path.clone()))
    }

    pub fn new_project(&mut self, project_path: &ProjectPath) -> Result<Project> {
        let project_id = 42;
        Ok(Project::new(project_id, project_path.clone()))
    }

    pub fn clean(&mut self) -> Result<()> {
        Ok(())
    }

    pub fn undo(&mut self) -> Result<()> {
        bail!("Cannot undo")
    }
}
