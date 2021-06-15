use anyhow::{bail, Result};

use std::collections::{HashMap, HashSet};

use crate::Repository;
use crate::{Project, RelativePath};

#[derive(Default, Debug)]
pub(crate) struct FakeRepository {
    global: HashSet<String>,
    by_extension: HashMap<String, Vec<String>>,
    by_project: HashMap<String, Vec<String>>,
    by_project_path: HashMap<(String, String), Vec<String>>,
    skip_file_names: HashSet<String>,
    skipped_paths: HashSet<(String, String)>,
    projects: Vec<String>,
}

impl FakeRepository {
    pub(crate) fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Repository for FakeRepository {
    fn project_exists(&self, path: &Project) -> Result<bool> {
        let index = &self.projects.iter().position(|x| x == &path.to_string());
        Ok(index.is_some())
    }

    fn new_project(&mut self, path: &Project) -> Result<()> {
        if self.project_exists(path)? {
            bail!("Project in '{}' already exists", path);
        }

        self.projects.push(path.to_string());
        Ok(())
    }

    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()> {
        for word in words {
            self.global.insert(word.to_string());
        }
        Ok(())
    }

    fn ignore(&mut self, word: &str) -> Result<()> {
        self.global.insert(word.to_string());
        Ok(())
    }

    fn is_ignored(&self, word: &str) -> Result<bool> {
        Ok(self.global.contains(word))
    }

    fn skip_file_name(&mut self, file_name: &str) -> Result<()> {
        self.skip_file_names.insert(file_name.to_string());
        Ok(())
    }

    fn is_skipped_file_name(&self, file_name: &str) -> Result<bool> {
        Ok(self.skip_file_names.contains(file_name))
    }

    fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let entry = &mut self
            .by_extension
            .entry(extension.to_string())
            .or_insert_with(Vec::new);
        entry.push(word.to_string());
        Ok(())
    }

    fn is_ignored_for_extension(&self, word: &str, extension: &str) -> Result<bool> {
        if let Some(words) = self.by_extension.get(extension) {
            Ok(words.contains(&word.to_string()))
        } else {
            Ok(false)
        }
    }

    fn ignore_for_project(&mut self, word: &str, project_path: &Project) -> Result<()> {
        let entry = &mut self
            .by_project
            .entry(project_path.to_string())
            .or_insert_with(Vec::new);
        entry.push(word.to_string());
        Ok(())
    }

    fn is_ignored_for_project(&self, word: &str, project_path: &Project) -> Result<bool> {
        if let Some(words) = self.by_project.get(&project_path.to_string()) {
            Ok(words.contains(&word.to_string()))
        } else {
            Ok(false)
        }
    }

    fn ignore_for_path(
        &mut self,
        word: &str,
        project_path: &Project,
        path: &RelativePath,
    ) -> Result<()> {
        let entry = &mut self
            .by_project_path
            .entry((project_path.to_string(), path.to_string()))
            .or_insert_with(Vec::new);
        entry.push(word.to_string());
        Ok(())
    }

    fn is_ignored_for_path(
        &self,
        word: &str,
        project_path: &Project,
        path: &RelativePath,
    ) -> Result<bool> {
        if let Some(words) = self
            .by_project_path
            .get(&(project_path.to_string(), path.to_string()))
        {
            Ok(words.contains(&word.to_string()))
        } else {
            Ok(false)
        }
    }

    fn skip_path(&mut self, project_path: &Project, path: &RelativePath) -> Result<()> {
        self.skipped_paths
            .insert((project_path.to_string(), path.to_string()));
        Ok(())
    }

    fn is_skipped_path(&self, project_path: &Project, path: &RelativePath) -> Result<bool> {
        Ok(self
            .skipped_paths
            .contains(&(project_path.to_string(), path.to_string())))
    }
}
