use anyhow::Result;
use std::path::Path;

use crate::{Operation, Project, ProjectId, ProjectPath, RelativePath};

pub struct ProjectInfo {
    id: ProjectId,
    path: String,
}

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
}
