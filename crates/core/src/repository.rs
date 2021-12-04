use std::path::Path;

use anyhow::Result;

pub use handler::{Operation, RepositoryHandler};

use crate::{Project, ProjectId, ProjectPath, RelativePath};

pub struct ProjectInfo {
    id: ProjectId,
    path: String,
}

pub mod handler;

// Note: the crucial difference with Project is that
// ProjectInfo does *not* contain the ProjectPath struct
// which is a NewType to represent *existing* project paths
//
// This is why this struct is only used in Repository::clean()
impl ProjectInfo {
    pub fn new(id: ProjectId, path: &str) -> Self {
        Self {
            id,
            path: path.to_string(),
        }
    }
    pub fn id(&self) -> ProjectId {
        self.id
    }

    pub fn path(&self) -> &str {
        &self.path
    }
}

pub trait Repository {
    // Add the list of words to the global ignore list
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()>;

    // Add word to the global ignore list
    fn ignore(&mut self, word: &str) -> Result<()>;
    // Is the word in the global ignore list?
    fn is_ignored(&self, word: &str) -> Result<bool>;

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
    // Is this file name to be skipped ?
    fn is_skipped_file_name(&self, file_name: &str) -> Result<bool>;

    // Add word to the ignore list for the given extension
    fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()>;
    // Is the word in the ignore list for the given extension?
    fn is_ignored_for_extension(&self, word: &str, extension: &str) -> Result<bool>;

    // Add word to the ignore list for the given project
    fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()>;
    // Is the word in the ignore list for the given project?
    fn is_ignored_for_project(&self, word: &str, project_id: ProjectId) -> Result<bool>;

    // Add word to the ignore list for the given project and path
    fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()>;
    // Is the word in the ignore list for the given project and path?
    fn is_ignored_for_path(
        &self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool>;

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
    // Is the given path in the given project to be skipped ?
    fn is_skipped_path(&self, project: ProjectId, relative_path: &RelativePath) -> Result<bool>;
    // Remove file name from the skip list
    fn unskip_file_name(&mut self, file_name: &str) -> Result<()>;
    // Remove relative file path from the skip list
    fn unskip_path(&mut self, project_id: ProjectId, relative_path: &RelativePath) -> Result<()>;

    // Should this file be skipped ?
    fn should_skip(&self, project_id: ProjectId, relative_path: &RelativePath) -> Result<bool> {
        if let Some(f) = relative_path.file_name() {
            if self.is_skipped_file_name(&f)? {
                return Ok(true);
            }
        }

        if self.is_skipped_path(project_id, relative_path)? {
            return Ok(true);
        }

        Ok(false)
    }

    // Insert a new operation
    fn insert_operation(&mut self, operation: &Operation) -> Result<()>;
    // Get last operation
    fn pop_last_operation(&mut self) -> Result<Option<Operation>>;

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
}