use anyhow::Result;
use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use skyspell_core::{Checker, Dictionary, IgnoreStore, Project, SpellingError};

#[derive(Debug, Serialize, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Default)]

pub(crate) struct SpellResult {
    errors: BTreeMap<String, Vec<Error>>,
    suggestions: BTreeMap<String, Vec<String>>,
}

pub struct JsonChecker<D: Dictionary> {
    project: Project,
    dictionary: D,
    ignore_store: IgnoreStore,
    unknown_words: BTreeSet<String>,
    spell_result: SpellResult,
}

impl<D: Dictionary> JsonChecker<D> {
    pub fn new(project: Project, dictionary: D, ignore_store: IgnoreStore) -> Result<Self> {
        Ok(Self {
            project,
            dictionary,
            ignore_store,
            spell_result: Default::default(),
            unknown_words: BTreeSet::new(),
        })
    }

    pub(crate) fn populate_result(&mut self) {
        let mut suggestions = BTreeMap::new();
        for word in &self.unknown_words {
            let suggestions_for_word = self.dictionary.suggest(word).unwrap_or_default();
            suggestions.insert(word.to_string(), suggestions_for_word);
        }

        self.spell_result.suggestions = suggestions;
    }

    pub(crate) fn result(&self) -> &SpellResult {
        &self.spell_result
    }
}

impl<D: Dictionary> Checker<D> for JsonChecker<D> {
    type SourceContext = ();

    fn dictionary(&self) -> &D {
        &self.dictionary
    }

    fn handle_error(
        &mut self,
        error: &SpellingError,
        _context: &Self::SourceContext,
    ) -> Result<()> {
        let (line, column) = (error.line(), error.column());
        let start_column = column + 1;
        let token = error.word();
        let project_file = error.project_file();
        let full_path = project_file.full_path();
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
        let entry = self
            .spell_result
            .errors
            .entry(full_path.to_string_lossy().to_string());
        let errors_for_entry = entry.or_default();
        errors_for_entry.push(error);
        self.unknown_words.insert(token.to_string());
        Ok(())
    }

    fn project(&self) -> &Project {
        &self.project
    }

    fn ignore_store(&mut self) -> &mut IgnoreStore {
        &mut self.ignore_store
    }

    fn success(&self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
