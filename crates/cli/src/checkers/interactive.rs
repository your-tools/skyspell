use std::collections::HashSet;

use anyhow::{bail, Result};
use colored::*;

use skyspell_core::Undoer;
use skyspell_core::{Checker, Dictionary, IgnoreStore};
use skyspell_core::{Project, RelativePath};

use crate::Interactor;
use crate::{info_2, print_error};

pub struct InteractiveChecker<I: Interactor, D: Dictionary, S: IgnoreStore> {
    project: Project,
    interactor: I,
    dictionary: D,
    undoer: Undoer<S>,
    skipped: HashSet<String>,
}

impl<I: Interactor, D: Dictionary, S: IgnoreStore> InteractiveChecker<I, D, S> {
    pub fn repository(&mut self) -> &mut S {
        self.undoer.ignore_store_mut()
    }
}

impl<I: Interactor, D: Dictionary, S: IgnoreStore> Checker for InteractiveChecker<I, D, S> {
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

    fn dictionary(&self) -> &dyn Dictionary {
        &self.dictionary
    }

    fn ignore_store(&self) -> &dyn IgnoreStore {
        self.undoer.ignore_store()
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
}

impl<I: Interactor, D: Dictionary, S: IgnoreStore> InteractiveChecker<I, D, S> {
    pub fn new(project: Project, interactor: I, dictionary: D, repository: S) -> Result<Self> {
        let undoer = Undoer::new(repository);
        Ok(Self {
            project,
            dictionary,
            interactor,
            undoer,
            skipped: HashSet::new(),
        })
    }

    fn on_error(&mut self, path: &RelativePath, pos: (usize, usize), error: &str) -> Result<()> {
        let (lineno, column) = pos;
        let prefix = format!("{}:{}:{}", path, lineno, column);
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
        self.undoer.ignore(error)?;
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

        self.undoer.ignore_for_extension(error, &extension)?;
        info_2!(
            "Added '{}' to the ignore list for extension '{}'",
            error,
            extension
        );
        Ok(true)
    }

    fn on_project_ignore(&mut self, error: &str) -> Result<bool> {
        self.undoer.ignore_for_project(error, self.project.id())?;
        info_2!(
            "Added '{}' to the ignore list for the current project",
            error
        );
        Ok(true)
    }

    fn on_file_ignore(&mut self, error: &str, relative_path: &RelativePath) -> Result<bool> {
        self.undoer
            .ignore_for_path(error, self.project.id(), relative_path)?;
        info_2!(
            "Added '{}' to the ignore list for path '{}'",
            error,
            relative_path
        );
        Ok(true)
    }

    pub fn ignore_store(&self) -> &dyn IgnoreStore {
        self.undoer.ignore_store()
    }
}

#[cfg(test)]
mod tests;
