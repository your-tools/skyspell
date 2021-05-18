use std::path::Path;

use anyhow::{bail, Result};

use crate::repo::AddFor;
use crate::Interactor;
use crate::Repo;

pub struct Handler<'repo, I: Interactor> {
    repo: &'repo dyn Repo,
    interactor: I,
}

impl<'repo, I: Interactor> Handler<'repo, I> {
    pub fn new(repo: &'repo dyn Repo, interactor: I) -> Self {
        Self { repo, interactor }
    }

    fn get_ext<'a>(&self, path: &'a Path) -> Option<&'a str> {
        let res = path.extension().or_else(|| {
            self.interactor.error("path has no extension");
            None
        })?;

        let res = res.to_str().or_else(|| {
            self.interactor.error("path extension is not utf-8");
            None
        })?;

        Some(res)
    }

    fn get_language_from_path(&self, path: &Path) -> Result<Option<i32>> {
        let ext = &self.get_ext(path);
        match ext {
            None => return Ok(None),
            Some(e) => match self.repo.lookup_extension(e)? {
                None => self.handle_new_extension(e),
                Some(r) => Ok(Some(r)),
            },
        }
    }

    fn handle_new_extension(&self, ext: &str) -> Result<Option<i32>> {
        let should_add = self
            .interactor
            .confirm(&format!("Add extension {} to the db (y/n)?", ext));
        if should_add {
            // Ask user to select between known languages
            todo!()
        }

        Ok(None)
    }

    pub fn handle(&mut self, path: &Path, pos: (usize, usize), error: &str) -> Result<()> {
        let (line, column) = pos;
        self.interactor
            .info(&format!("{}:{}:{} {}", path.display(), line, column, error));
        let prompt = r#"
        Add to (n)atural language ignore list
        Add to (p)rogramming language ignore list
        Ignore just for this (f)ile
        (q)uit

        What to do?
        "#;

        let answer = self.interactor.input_letter(prompt, "npfq");
        let add_for = match answer.as_ref() {
            "n" => AddFor::NaturalLanguage,
            "p" => {
                let id = self.get_language_from_path(&path)?;
                match id {
                    None => todo!(),
                    Some(i) => AddFor::ProgrammingLanguage(i),
                }
            }
            "f" => AddFor::File(2),
            "q" => {
                bail!("Interrupted by user");
            }
            _ => {
                unreachable!()
            }
        };

        self.repo.add_word(error, &add_for)
    }
}
