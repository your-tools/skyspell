use anyhow::Result;

use crate::{IgnoreStore, Operation, ProjectId, ProjectInfo, ProjectPath};

pub trait Repository {
    fn as_ignore_store(&mut self) -> &mut dyn IgnoreStore;
    fn clean(&mut self) -> Result<()> {
        todo!()
    }

    /// Add a new project
    fn new_project(&mut self, project_path: &ProjectPath) -> Result<ProjectId>;
    /// Check if a project exists
    fn project_exists(&self, project_path: &ProjectPath) -> Result<bool>;
    /// Remove the given project from the list
    fn remove_project(&mut self, project_id: ProjectId) -> Result<()>;
    /// Get project id
    fn get_project_id(&self, project_path: &ProjectPath) -> Result<ProjectId>;
    /// Get the list of known projects. Used for cleanup
    fn projects(&self) -> Result<Vec<ProjectInfo>>;

    // Insert a new operation
    fn insert_operation(&mut self, operation: &Operation) -> Result<()>;
    // Get last operation
    fn pop_last_operation(&mut self) -> Result<Option<Operation>>;
}
