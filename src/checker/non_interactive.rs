use anyhow::{bail, Result};
use colored::*;

use crate::Checker;
use crate::Dictionary;
use crate::Repository;
use crate::{ProjectId, ProjectPath, RelativePath};

pub(crate) struct NonInteractiveChecker<D: Dictionary, R: Repository> {
    project: ProjectPath,
    project_id: ProjectId,
    dictionary: D,
    repository: R,
    errors_found: bool,
}

impl<D: Dictionary, R: Repository> NonInteractiveChecker<D, R> {
    pub(crate) fn new(project: ProjectPath, dictionary: D, mut repository: R) -> Result<Self> {
        let project_id = repository.ensure_project(&project)?;
        Ok(Self {
            project,
            project_id,
            dictionary,
            repository,
            errors_found: false,
        })
    }
}

impl<D: Dictionary, R: Repository> Checker for NonInteractiveChecker<D, R> {
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
        println!("{} {}", prefix.bold(), token.blue());
        Ok(())
    }

    fn success(&self) -> Result<()> {
        if self.errors_found {
            bail!("Found spelling errors");
        }
        Ok(())
    }

    fn project(&self) -> &ProjectPath {
        &self.project
    }

    fn project_id(&self) -> ProjectId {
        self.project_id
    }

    fn repository(&self) -> &dyn Repository {
        &self.repository
    }
}
