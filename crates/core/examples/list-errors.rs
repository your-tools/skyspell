use std::path::Path;

use anyhow::{bail, Result};
use skyspell_core::{
    global_path, Checker, IgnoreStore, TokenProcessor, SKYSPELL_LOCAL_IGNORE,
};
use skyspell_core::{EnchantDictionary, Project};

struct SimpleChecker {
    dictionary: EnchantDictionary,
    project: Project,
    ignore_store: IgnoreStore,
    error_count: usize,
}

impl SimpleChecker {
    fn try_new() -> Result<Self> {
        let dictionary = EnchantDictionary::new("en_US")?;
        let project_path = Path::new(".");
        let local_path = project_path.join(SKYSPELL_LOCAL_IGNORE);
        let project = Project::new(project_path)?;
        let global_path = global_path()?;

        let ignore_store = IgnoreStore::load(global_path, local_path)?;
        Ok(Self {
            dictionary,
            project,
            ignore_store,
            error_count: 0,
        })
    }
}

impl Checker<EnchantDictionary> for SimpleChecker {
    type Context = (usize, usize); // line, column

    fn dictionary(&self) -> &EnchantDictionary {
        &self.dictionary
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn success(&self) -> Result<()> {
        if self.error_count != 0 {
            bail!("Found some errors");
        }
        Ok(())
    }

    fn ignore_store(&mut self) -> &mut IgnoreStore {
        &mut self.ignore_store
    }

    fn handle_error(
        &mut self,
        error: &str,
        path: &skyspell_core::RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        let (line, column) = context;
        println!("{path}:{line}:{column} {error}");
        self.error_count += 1;
        Ok(())
    }
}

fn main() -> Result<()> {
    let mut checker = SimpleChecker::try_new()?;
    let source_path = Path::new("README.md");
    let token_processor = TokenProcessor::new(source_path);
    let relative_path = checker.to_relative_path(source_path)?;
    token_processor.each_token(|token, line, column| {
        checker.handle_token(token, &relative_path, &(line, column))
    })?;
    checker.success()?;
    Ok(())
}
