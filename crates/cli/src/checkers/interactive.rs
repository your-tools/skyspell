use crate::{info_1, Interactor};
use crate::{info_2, print_error};
use anyhow::{bail, Result};
use colored::*;
use skyspell_core::Operation;
use skyspell_core::{Checker, CheckerState, Config, Dictionary};
use skyspell_core::{Project, RelativePath};
use std::collections::HashSet;

pub struct InteractiveChecker<I: Interactor, D: Dictionary> {
    project: Project,
    interactor: I,
    dictionary: D,
    ignore_config: Config,
    state: CheckerState,
    skipped: HashSet<String>,
}

impl<I: Interactor, D: Dictionary> Checker<D> for InteractiveChecker<I, D> {
    // line, column
    type Context = (usize, usize);

    fn success(&self) -> Result<()> {
        if !self.skipped.is_empty() {
            bail!("Some errors were skipped")
        } else {
            info_2!("No errors found");
            Ok(())
        }
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn dictionary(&self) -> &D {
        &self.dictionary
    }

    fn ignore_config(&mut self) -> &mut Config {
        &mut self.ignore_config
    }

    fn state(&mut self) -> Option<&mut CheckerState> {
        Some(&mut self.state)
    }

    fn handle_error(
        &mut self,
        error: &str,
        path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        let &(line, column) = context;
        if self.skipped.contains(error) {
            return Ok(());
        }
        self.on_error(path, (line, column), error)
    }

    fn apply_operation(&mut self, mut operation: Operation) -> Result<()> {
        operation.execute(&mut self.ignore_config)?;
        self.state.set_last_operation(operation.clone())
    }
}

impl<I: Interactor, D: Dictionary> InteractiveChecker<I, D> {
    pub fn new(
        project: Project,
        interactor: I,
        dictionary: D,
        ignore_config: Config,
    ) -> Result<Self> {
        info_1!(
            "Checking project {} for spelling errors",
            project.path().as_str().bold()
        );
        let state = CheckerState::load()?;
        Ok(Self {
            project,
            dictionary,
            interactor,
            ignore_config,
            skipped: HashSet::new(),
            state,
        })
    }

    fn on_error(&mut self, path: &RelativePath, pos: (usize, usize), error: &str) -> Result<()> {
        let (lineno, column) = pos;
        let prefix = format!("{path}:{lineno}:{column}");
        println!("{} {}", prefix, error.red());
        let prompt = r#"What to do?
a : Add word to global ignore list
e : Add word to ignore list for this extension
p : Add word to ignore list for the current project
f : Add word to ignore list for the current file
x : Skip this error
q : Quit
> "#;

        loop {
            let letter = self.interactor.input_letter(prompt, "aepfnsxq");
            match letter.as_ref() {
                "a" => {
                    if self.on_global_ignore(error)? {
                        break;
                    }
                }
                "e" => {
                    if self.on_extension(path, error)? {
                        break;
                    }
                }
                "p" => {
                    if self.on_project_ignore(error)? {
                        break;
                    }
                }
                "f" => {
                    if self.on_file_ignore(error, path)? {
                        break;
                    }
                }
                "q" => {
                    bail!("Interrupted by user")
                }
                "x" => {
                    self.skipped.insert(error.to_string());
                    break;
                }
                _ => {
                    unreachable!()
                }
            }
        }
        Ok(())
    }

    // Note: this cannot fail, but it's convenient to have it return a
    // boolean like the other on_* methods
    fn on_global_ignore(&mut self, error: &str) -> Result<bool> {
        let operation = Operation::new_ignore(error);
        self.apply_operation(operation)?;
        info_2!("Added '{}' to the global ignore list", error);
        Ok(true)
    }

    fn on_extension(&mut self, relative_path: &RelativePath, error: &str) -> Result<bool> {
        let extension = match relative_path.extension() {
            None => {
                print_error!("{} has no extension", relative_path);
                return Ok(false);
            }
            Some(e) => e,
        };

        let operation = Operation::new_ignore_for_extension(error, &extension);
        self.apply_operation(operation)?;
        info_2!(
            "Added '{}' to the ignore list for extension '{}'",
            error,
            extension
        );
        Ok(true)
    }

    fn on_project_ignore(&mut self, error: &str) -> Result<bool> {
        let operation = Operation::new_ignore_for_project(error);
        self.apply_operation(operation)?;
        info_2!(
            "Added '{}' to the ignore list for the current project",
            error
        );
        Ok(true)
    }

    fn on_file_ignore(&mut self, error: &str, relative_path: &RelativePath) -> Result<bool> {
        let operation = Operation::new_ignore_for_path(error, relative_path);
        self.apply_operation(operation)?;
        info_2!(
            "Added '{}' to the ignore list for path '{}'",
            error,
            relative_path
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
