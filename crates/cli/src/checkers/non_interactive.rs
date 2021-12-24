use anyhow::{bail, Result};
use colored::*;

use skyspell_core::{Checker, Dictionary, IgnoreStore};
use skyspell_core::{Project, RelativePath};

pub struct NonInteractiveChecker<D: Dictionary, I: IgnoreStore> {
    project: Project,
    dictionary: D,
    ignore_store: I,
    num_errors: usize,
}

impl<D: Dictionary, I: IgnoreStore> NonInteractiveChecker<D, I> {
    pub fn new(project: Project, dictionary: D, ignore_store: I) -> Result<Self> {
        Ok(Self {
            project,
            dictionary,
            ignore_store,
            num_errors: 0,
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
        self.num_errors += 1;
        let prefix = format!("{}:{}:{}", path, line, column);
        println!(
            "{}: {}: {}: {}",
            prefix,
            "error".red(),
            "unknown word".clear(),
            token
        );
        Ok(())
    }

    fn success(&self) -> Result<()> {
        match self.num_errors {
            0 => {
                println!("Success! No spelling errors found");
                Ok(())
            }
            1 => bail!("Found just one tiny spelling error"),
            n => bail!("Found {} spelling errors", n),
        }
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn ignore_store(&self) -> &dyn IgnoreStore {
        &self.ignore_store
    }
}
