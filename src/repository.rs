use std::path::Path;

use anyhow::Result;

use crate::Ignore;
use crate::{Project, ProjectId, ProjectPath, RelativePath};

pub struct ProjectInfo {
    id: ProjectId,
    path: String,
}

pub(crate) mod handler;
pub use handler::{Operation, RepositoryHandler};

// Note: the crucial difference with Project is that
// ProjectInfo does *not* contain the ProjectPath struct
// which is a NewType to represent *existing* project paths
//
// This is why this struct is only used in Repository::clean()
impl ProjectInfo {
    pub(crate) fn new(id: ProjectId, path: &str) -> Self {
        Self {
            id,
            path: path.to_string(),
        }
    }
    pub(crate) fn id(&self) -> ProjectId {
        self.id
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }
}

pub trait Repository: Ignore {
    // Add the list of words to the global ignore list
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()>;

    // Add word to the global ignore list
    fn ignore(&mut self, word: &str) -> Result<()>;

    // Add a new project
    fn new_project(&mut self, project_path: &ProjectPath) -> Result<ProjectId>;
    // Check if a project exists
    fn project_exists(&self, project_path: &ProjectPath) -> Result<bool>;
    // Create a project if it does not exist yet
    fn ensure_project(&mut self, project_path: &ProjectPath) -> Result<Project> {
        if !self.project_exists(project_path)? {
            self.new_project(project_path)?;
        }
        let id = self.get_project_id(project_path)?;
        Ok(Project::new(id, project_path.clone()))
    }

    // Remove the given project from the list
    fn remove_project(&mut self, project_id: ProjectId) -> Result<()>;
    // Get project id
    fn get_project_id(&self, project_path: &ProjectPath) -> Result<ProjectId>;
    fn projects(&self) -> Result<Vec<ProjectInfo>>;

    fn clean(&mut self) -> Result<()> {
        for project in self.projects()? {
            let path = project.path();
            let path = Path::new(&path);
            let id = project.id();
            if !path.exists() {
                self.remove_project(id)?;
                println!("Removed non longer existing project: {}", path.display());
            }
        }
        Ok(())
    }

    // Always skip this file name - to be used with Cargo.lock, yarn.lock
    // and the like
    fn skip_file_name(&mut self, file_name: &str) -> Result<()>;

    // Add word to the ignore list for the given extension
    fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()>;

    // Add word to the ignore list for the given project
    fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()>;

    // Add word to the ignore list for the given project and path
    fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()>;

    // Remove word from the global ignore list
    fn remove_ignored(&mut self, word: &str) -> Result<()>;
    // Remove word from the ignore list for the given extension
    fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<()>;
    // Remove word from the ignore list for the given path
    fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()>;
    // Remove word from the ignore list for the given project
    fn remove_ignored_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()>;

    // Always skip the given file for the given project
    fn skip_path(&mut self, project_id: ProjectId, relative_path: &RelativePath) -> Result<()>;
    // Remove file name from the skip list
    fn unskip_file_name(&mut self, file_name: &str) -> Result<()>;
    // Remove relative file path from the skip list
    fn unskip_path(&mut self, project_id: ProjectId, relative_path: &RelativePath) -> Result<()>;

    // Insert a new operation
    fn insert_operation(&mut self, operation: &Operation) -> Result<()>;
    // Get last operation
    fn pop_last_operation(&mut self) -> Result<Option<Operation>>;
}

#[cfg(test)]
mod tests;
