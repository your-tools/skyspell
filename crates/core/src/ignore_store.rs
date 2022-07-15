use anyhow::Result;
use std::path::Path;

use anyhow::Result;

pub use handler::{Operation, RepositoryHandler};

use crate::IgnoreStore;
use crate::{Project, ProjectId, ProjectPath, RelativePath};

pub struct ProjectInfo {
    id: ProjectId,
    path: String,
}

use crate::{ProjectId, RelativePath};

pub trait IgnoreStore {
    // Is the word in the global ignore list?
    fn is_ignored(&self, word: &str) -> Result<bool>;

    // Is the word in the ignore list for the given extension?
    fn is_ignored_for_extension(&self, word: &str, extension: &str) -> Result<bool>;

    // Is the word in the ignore list for the given project?
    fn is_ignored_for_project(&self, word: &str, project_id: ProjectId) -> Result<bool>;

    // Is the word in the ignore list for the given project and path?
    fn is_ignored_for_path(
        &self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool>;

    // Should this word be ignored?
    // This is called when a word is *not* found in the spelling dictionary.
    //
    // A word is ignored if:
    //   * it's in the global ignore list
    //   * the relative path has an extension and it's in the ignore list
    //     for this extension
    //   * it's in the ignore list for the project
    //   * it's in the ignore list for the relative path
    //
    // Otherwise, it's *not* ignored and the Checker will call handle_error()
    //
    fn should_ignore(
        &self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        if self.is_ignored(word)? {
            return Ok(true);
        }

        if let Some(e) = relative_path.extension() {
            if self.is_ignored_for_extension(word, &e)? {
                return Ok(true);
            }
        }

        if self.is_ignored_for_project(word, project_id)? {
            return Ok(true);
        }

        self.is_ignored_for_path(word, project_id, relative_path)
    }

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

    // Insert a new operation
    fn insert_operation(&mut self, operation: &Operation) -> Result<()>;
    // Get last operation
    fn pop_last_operation(&mut self) -> Result<Option<Operation>>;
}
