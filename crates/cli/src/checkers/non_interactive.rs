use anyhow::{bail, Result};
use colored::*;

use skyspell_core::{Checker, Dictionary, IgnoreStore, StorageBackend};
use skyspell_core::{Project, RelativePath};

use crate::info_2;

pub struct NonInteractiveChecker<D: Dictionary> {
    project: Project,
    dictionary: D,
    storage_backend: StorageBackend,
    num_errors: usize,
}

impl<D: Dictionary> NonInteractiveChecker<D> {
    pub fn new(project: Project, dictionary: D, storage_backend: StorageBackend) -> Result<Self> {
        Ok(Self {
            project,
            dictionary,
            storage_backend,
            num_errors: 0,
        })
    }
}

impl<D: Dictionary> Checker for NonInteractiveChecker<D> {
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
        let prefix = format!("{}:{}:{}", path, line, column + 1);
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
                info_2!("Success! No spelling errors found");
                Ok(())
            }
            1 => bail!("Found just one tiny spelling error"),
            n => bail!("Found {} spelling errors", n),
        }
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn storage_backend(&self) -> &StorageBackend {
        &self.storage_backend
    }
}
