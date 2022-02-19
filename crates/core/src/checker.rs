use std::path::Path;

use anyhow::Result;

use crate::{Dictionary, IgnoreStore};
use crate::{Project, RelativePath};

pub trait Checker {
    type Context;

    fn handle_error(
        &mut self,
        error: &str,
        path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()>;

    // Were all the errors handled properly?
    fn success(&self) -> Result<()>;

    fn ignore_store(&self) -> &dyn IgnoreStore;

    fn dictionary(&self) -> &dyn Dictionary;

    fn project(&self) -> &Project;

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
        let ignore_store = self.ignore_store();
        let project_id = self.project().id();
        let should_ignore = ignore_store.should_ignore(token, project_id, relative_path)?;
        if !should_ignore {
            self.handle_error(token, relative_path, context)?
        }
        Ok(())
    }
}
