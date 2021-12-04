use anyhow::{bail, Result};
use colored::*;

use skyspell_core::Checker;
use skyspell_core::Dictionary;
use skyspell_core::Repository;
use skyspell_core::{Project, ProjectPath, RelativePath};

pub struct NonInteractiveChecker<D: Dictionary, R: Repository> {
    project: Project,
    dictionary: D,
    repository: R,
    errors_found: bool,
}

impl<D: Dictionary, R: Repository> NonInteractiveChecker<D, R> {
    pub fn new(project_path: ProjectPath, dictionary: D, mut repository: R) -> Result<Self> {
        let project = repository.ensure_project(&project_path)?;
        Ok(Self {
            project,
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

    fn repository(&self) -> &dyn Repository {
        &self.repository
    }
}