use anyhow::{anyhow, Result};

use crate::{IgnoreStore, Operation, ProjectId, ProjectInfo, ProjectPath};

pub trait Repository {
    fn ignore_store_mut(&mut self) -> &mut dyn IgnoreStore;
    fn ignore_store(&self) -> &dyn IgnoreStore;

    /// Add a new project
    fn new_project(&mut self, project_path: &ProjectPath) -> Result<ProjectId>;
    /// Check if a project exists
    fn project_exists(&mut self, project_path: &ProjectPath) -> Result<bool>;
    /// Remove the given project from the list
    fn remove_project(&mut self, project_id: ProjectId) -> Result<()>;
    /// Get project id
    fn get_project_id(&mut self, project_path: &ProjectPath) -> Result<ProjectId>;
    /// Get the list of known projects. Used for cleanup
    fn projects(&mut self) -> Result<Vec<ProjectInfo>>;

    /// Insert a new operation
    fn insert_operation(&mut self, operation: &Operation) -> Result<()>;
    /// Get last operation
    fn pop_last_operation(&mut self) -> Result<Option<Operation>>;

    /// Undo last operation
    fn undo(&mut self) -> Result<()> {
        let last_operation = self.pop_last_operation()?;
        let mut last_operation = last_operation.ok_or_else(|| anyhow!("Nothing to undo"))?;
        last_operation.undo(self.ignore_store_mut())
    }
}
