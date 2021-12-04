use anyhow::{bail, Result};
use colored::*;

use skyspell_core::{Checker, Dictionary, IgnoreStore};
use skyspell_core::{Project, RelativePath};

pub struct NonInteractiveChecker<D: Dictionary, I: IgnoreStore> {
    project: Project,
    dictionary: D,
    ignore_store: I,
    errors_found: bool,
}

impl<D: Dictionary, I: IgnoreStore> NonInteractiveChecker<D, I> {
    pub fn new(project: Project, dictionary: D, ignore_store: I) -> Result<Self> {
        Ok(Self {
            project,
            dictionary,
            ignore_store,
            errors_found: false,
        })
    }
}

impl<D: Dictionary, I: IgnoreStore> Checker for NonInteractiveChecker<D, I> {
    // line, column
    type Context = (usize, usize);

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn handle_error(
        &mut self,
        token: &str,
        path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        let &(line, column) = context;
        self.errors_found = true;
        let prefix = format!("{}:{}:{}", path, line, column);
        println!("{} {}", prefix, token.red());
        Ok(())
    }

    fn success(&self) -> Result<()> {
        if self.errors_found {
            bail!("Found spelling errors");
        }
        Ok(())
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn ignore_store(&self) -> &dyn IgnoreStore {
        &self.ignore_store
    }
}
