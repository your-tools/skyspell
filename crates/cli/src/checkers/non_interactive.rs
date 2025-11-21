use crate::{CheckOpts, OutputFormat, info_1, info_2};
use anyhow::{Result, bail};
use colored::*;
use skyspell_core::Project;
use skyspell_core::{Checker, Dictionary, IgnoreStore, Operation, SpellingError};

pub struct NonInteractiveChecker<D: Dictionary> {
    project: Project,
    dictionary: D,
    ignore_store: IgnoreStore,
    output_format: OutputFormat,
    num_errors: usize,
}

impl<D: Dictionary> NonInteractiveChecker<D> {
    pub fn new(
        project: Project,
        dictionary: D,
        ignore_store: IgnoreStore,
        opts: &CheckOpts,
    ) -> Result<Self> {
        let output_format = opts.output_format.unwrap_or_default();
        if output_format.is_text() {
            info_1!(
                "Checking project {} for spelling errors",
                project.path_string().bold()
            );
        }
        Ok(Self {
            project,
            dictionary,
            ignore_store,
            output_format,
            num_errors: 0,
        })
    }

    fn print_error(&self, error: &SpellingError) {
        let SpellingError {
            word,
            source_path,
            pos,
        } = error;
        let (line, col) = pos;
        let path = source_path.to_string_lossy();
        let prefix = format!("{path}:{line}:{col}");
        match self.output_format {
            OutputFormat::Text => println!(
                "{}: {}: {}: {}",
                prefix,
                "error".red(),
                "unknown word".clear(),
                word
            ),
            OutputFormat::Json => {}
        }
    }
}

impl<D: Dictionary> Checker<D> for NonInteractiveChecker<D> {
    type SourceContext = ();

    fn dictionary(&self) -> &D {
        &self.dictionary
    }

    fn handle_error(
        &mut self,
        error: &SpellingError,
        _context: &Self::SourceContext,
    ) -> Result<()> {
        self.num_errors += 1;
        self.print_error(error);
        Ok(())
    }

    fn success(&self) -> Result<()> {
        match self.num_errors {
            0 => {
                info_2!("Success! No spelling errors found");
                Ok(())
            }
            1 => bail!("Found just one tiny spelling error"),
            n => bail!("Found {n} spelling errors"),
        }
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn ignore_store(&mut self) -> &mut IgnoreStore {
        &mut self.ignore_store
    }

    fn apply_operation(&mut self, mut operation: Operation) -> Result<()> {
        operation.execute(&mut self.ignore_store)
    }
}
