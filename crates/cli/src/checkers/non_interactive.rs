use crate::{info_1, info_2, OutputFormat};
use anyhow::{bail, Result};
use colored::*;
use serde::Serialize;
use skyspell_core::{Checker, Config, Dictionary, Operation};
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
    ignore_config: Config,
    output_format: OutputFormat,
    errors: BTreeMap<String, Vec<Error>>,
    num_errors: usize,
}

impl<D: Dictionary> NonInteractiveChecker<D> {
    pub fn new(
        project: Project,
        dictionary: D,
        ignore_config: Config,
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
            ignore_config,
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
            n => bail!("Found {} spelling errors", n),
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
    // line, column
    type Context = (usize, usize);

    fn dictionary(&self) -> &D {
        &self.dictionary
    }

    fn handle_error(
        &mut self,
        token: &str,
        path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        self.num_errors += 1;
        let &(line, column) = context;
        let start_column = column + 1;
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
            self.print_error(path, &error);
        }
        let entry = self.errors.entry(path.to_string());
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

    fn ignore_config(&mut self) -> &mut Config {
        &mut self.ignore_config
    }

    fn apply_operation(&mut self, mut operation: Operation) -> Result<()> {
        operation.execute(&mut self.ignore_config)
    }
}
