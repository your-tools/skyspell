use std::path::Path;

use anyhow::Result;

use crate::Dictionary;
use crate::Repository;
use crate::{Project, RelativePath};

mod interactive;
mod non_interactive;

pub(crate) use interactive::InteractiveChecker;
pub(crate) use non_interactive::NonInteractiveChecker;

pub(crate) trait Checker {
    type Context;

    fn handle_error(
        &mut self,
        error: &str,
        path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()>;

    // Were all the errors handled properly?
    fn success(&self) -> Result<()>;
    fn repository(&self) -> &dyn Repository;
    fn dictionary(&self) -> &dyn Dictionary;

    fn project(&self) -> &Project;

    fn should_skip(&self, path: &RelativePath) -> Result<bool> {
        let repository = self.repository();
        let project_id = self.project().id();
        repository.should_skip(project_id, path)
    }

    fn to_relative_path(&self, path: &Path) -> Result<RelativePath> {
        let project_path = self.project().path();
        RelativePath::new(project_path, path)
    }

    fn handle_token(
        &mut self,
        token: &str,
        relative_path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        let dictionary = self.dictionary();
        let in_dict = dictionary.check(token)?;
        if in_dict {
            return Ok(());
        }
        let repository = self.repository();
        let project_id = self.project().id();
        let should_ignore = repository.should_ignore(token, project_id, relative_path)?;
        if !should_ignore {
            self.handle_error(token, relative_path, context)?
        }
        Ok(())
    }
}
