use crate::{info_1, info_2, OutputFormat};
use anyhow::{bail, Result};
use colored::*;
use serde::Serialize;
use skyspell_core::{Checker, Dictionary, IgnoreStore, Operation, SpellingError};
use skyspell_core::{Project, RelativePath};
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
struct Range {
    line: usize,
    start_column: usize,
    end_column: usize,
}

#[derive(Debug, Serialize)]
struct Error {
    word: String,
    range: Range,
}

pub struct NonInteractiveChecker<D: Dictionary> {
    project: Project,
    dictionary: D,
    ignore_store: IgnoreStore,
    output_format: OutputFormat,
    errors: BTreeMap<String, Vec<Error>>,
    num_errors: usize,
}

impl<D: Dictionary> NonInteractiveChecker<D> {
    pub fn new(
        project: Project,
        dictionary: D,
        ignore_store: IgnoreStore,
        output_format: OutputFormat,
    ) -> Result<Self> {
        if output_format.is_text() {
            info_1!(
                "Checking project {} for spelling errors",
                project.path().as_str().bold()
            );
        }
        Ok(Self {
            project,
            dictionary,
            ignore_store,
            output_format,
            errors: BTreeMap::new(),
            num_errors: 0,
        })
    }

    fn print_error(&self, path: &RelativePath, error: &Error) {
        let Error { range, word } = error;
        let Range {
            line,
            start_column,
            end_column,
        } = range;
        let prefix = format!("{path}:{line}:{start_column}:{end_column}");
        println!(
            "{}: {}: {}: {}",
            prefix,
            "error".red(),
            "unknown word".clear(),
            word
        );
    }

    fn success_text(&self) -> Result<()> {
        match self.num_errors {
            0 => {
                info_2!("Success! No spelling errors found");
                Ok(())
            }
            1 => bail!("Found just one tiny spelling error"),
            n => bail!("Found {n} spelling errors"),
        }
    }

    fn success_json(&self) -> Result<()> {
        let json = serde_json::to_string(&self.errors).expect("errors should be serializable");
        println!("{json}");
        if self.errors.is_empty() {
            Ok(())
        } else {
            bail!("Found some errors");
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
        let (line, column) = error.pos();
        let start_column = column + 1;
        let token = error.word();
        let path = error.relative_path();
        let end_column = start_column + token.chars().count() - 1;
        let range = Range {
            line,
            start_column,
            end_column,
        };
        let error = Error {
            word: token.to_string(),
            range,
        };
        if self.output_format == OutputFormat::Text {
            self.print_error(&path, &error);
        }
        let entry = self.errors.entry(path.normalize());
        let errors_for_entry = entry.or_default();
        errors_for_entry.push(error);
        Ok(())
    }

    fn success(&self) -> Result<()> {
        match self.output_format {
            OutputFormat::Text => self.success_text(),
            OutputFormat::Json => self.success_json(),
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

#[cfg(test)]
mod tests;
