use anyhow::Result;

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
}
