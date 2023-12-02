use anyhow::{bail, Result};

use crate::operations::Operation;
use crate::{IgnoreConfig, Project, ProjectId, ProjectPath, RelativePath};

pub struct StorageBackend(IgnoreConfig);

impl StorageBackend {
    pub fn new(ignore_config: IgnoreConfig) -> Self {
        Self(ignore_config)
    }

    pub(crate) fn should_ignore(
        &mut self,
        token: &str,
        project_id: i32,
        relative_path: &crate::RelativePath,
    ) -> Result<bool> {
        self.0.should_ignore(token, project_id, relative_path)
    }

    pub fn is_ignored(&mut self, word: &str) -> Result<bool> {
        self.0.is_ignored(word)
    }

    pub fn is_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<bool> {
        self.0.is_ignored_for_extension(word, ext)
    }

    pub fn is_ignored_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<bool> {
        self.0.is_ignored_for_project(word, project_id)
    }

    pub fn is_ignored_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        self.0.is_ignored_for_path(word, project_id, relative_path)
    }

    pub fn ignore(&mut self, word: &str) -> Result<()> {
        let _operation = Operation::new_ignore(word);
        self.0.ignore(word)
    }

    pub fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        let _operation = Operation::new_ignore_for_project(word, project_id);
        self.0.ignore_for_project(word, project_id)
    }

    pub fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let _operation = Operation::new_ignore_for_path(word, project_id, relative_path);
        self.0.ignore_for_path(word, project_id, relative_path)
    }

    pub fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let _operation = Operation::new_ignore_for_extension(word, extension);
        self.0.ignore_for_extension(word, extension)
    }

    pub fn remove_ignored(&mut self, word: &str) -> Result<()> {
        self.0.remove_ignored(word)
    }

    pub fn remove_ignored_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        self.0.remove_ignored_for_project(word, project_id)
    }

    pub fn remove_ignored_for_extension(&mut self, word: &str, ext: &str) -> Result<()> {
        self.0.remove_ignored_for_extension(word, ext)
    }

    pub fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()> {
        self.0
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
