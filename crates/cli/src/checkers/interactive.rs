use crate::{Interactor, info_1};
use crate::{info_2, print_error};
use anyhow::{Result, bail};
use colored::*;
use skyspell_core::{Checker, CheckerState, Dictionary, SpellingError};
use skyspell_core::{IgnoreStore, Operation};
use skyspell_core::{Project, ProjectFile};
use std::collections::HashSet;
use std::path::PathBuf;

pub struct InteractiveChecker<I: Interactor, D: Dictionary> {
    project: Project,
    interactor: I,
    dictionary: D,
    ignore_store: IgnoreStore,
    state: CheckerState,
    skipped: HashSet<String>,
}

impl<I: Interactor, D: Dictionary> Checker<D> for InteractiveChecker<I, D> {
    type SourceContext = ();

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

    fn ignore_store(&mut self) -> &mut IgnoreStore {
        &mut self.ignore_store
    }

    fn state(&mut self) -> Option<&mut CheckerState> {
        Some(&mut self.state)
    }

    fn handle_error(
        &mut self,
        error: &SpellingError,
        _context: &Self::SourceContext,
    ) -> Result<()> {
        let (line, column) = error.pos();
        let word = error.word();
        if self.skipped.contains(word) {
            return Ok(());
        }
        let project_file = error.project_file();
        self.on_error(project_file, (line, column), word)
    }

    fn apply_operation(&mut self, mut operation: Operation) -> Result<()> {
        operation.execute(&mut self.ignore_store)?;
        self.state.set_last_operation(operation.clone())
    }
}

impl<I: Interactor, D: Dictionary> InteractiveChecker<I, D> {
    pub fn new(
        project: Project,
        interactor: I,
        dictionary: D,
        ignore_store: IgnoreStore,
        state_toml: Option<PathBuf>,
    ) -> Result<Self> {
        info_1!(
            "Checking project {} for spelling errors",
            project.path_string().bold()
        );
        let state = CheckerState::load(state_toml)?;
        Ok(Self {
            project,
            dictionary,
            interactor,
            ignore_store,
            skipped: HashSet::new(),
            state,
        })
    }

    fn on_error(
        &mut self,
        project_file: &ProjectFile,
        pos: (usize, usize),
        error: &str,
    ) -> Result<()> {
        let lang = self.dictionary().lang().to_owned();
        let (lineno, column) = pos;
        let file_name = project_file.name();
        let prefix = format!("{file_name}:{lineno}:{column}");
        println!("{} {}", prefix, error.bold().red());
        let prompt = r#"What to do?
g : Add word to global ignore list
l : Add word to the ignore list for the current language
e : Add word to ignore list for this extension
p : Add word to ignore list for the current project
f : Add word to ignore list for the current file
x : Skip this error
q : Quit
> "#;

        loop {
            let letter = self.interactor.input_letter(prompt, "glepfnsxq");
            match letter.as_ref() {
                "g" => {
                    if self.on_global_ignore(error)? {
                        break;
                    }
                }
                "l" => {
                    if self.on_lang(error, &lang)? {
                        break;
                    }
                }
                "e" => {
                    if self.on_extension(project_file, error)? {
                        break;
                    }
                }
                "p" => {
                    if self.on_project_ignore(error)? {
                        break;
                    }
                }
                "f" => {
                    if self.on_file_ignore(error, project_file)? {
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

    fn on_global_ignore(&mut self, error: &str) -> Result<bool> {
        let operation = Operation::new_ignore(error);
        self.apply_operation(operation)?;
        info_2!("Added '{}' to the global ignore list", error);
        Ok(true)
    }

    fn on_extension(&mut self, project_file: &ProjectFile, error: &str) -> Result<bool> {
        let extension = match project_file.extension() {
            None => {
                print_error!("{} has no extension", project_file.name());
                return Ok(false);
            }
            Some(e) => e,
        };

        let operation = Operation::new_ignore_for_extension(error, extension);
        self.apply_operation(operation)?;
        info_2!(
            "Added '{}' to the ignore list for extension '{}'",
            error,
            extension
        );
        Ok(true)
    }

    fn on_lang(&mut self, error: &str, lang: &str) -> Result<bool> {
        let operation = Operation::new_ignore_for_lang(error, lang);
        self.apply_operation(operation)?;
        info_2!("Added '{}' to the ignore list for '{}'", error, lang);
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

    fn on_file_ignore(&mut self, error: &str, project_file: &ProjectFile) -> Result<bool> {
        let operation = Operation::new_ignore_for_path(error, project_file);
        self.apply_operation(operation)?;
        info_2!(
            "Added '{}' to the ignore list for path '{}'",
            error,
            project_file.name()
        );
        Ok(true)
    }
}

#[cfg(test)]
mod tests;
