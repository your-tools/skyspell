use anyhow::{anyhow, bail, ensure, Result};

use std::collections::{HashMap, HashSet};

use crate::IgnoreStore;
use crate::Operation;
use crate::ProjectInfo;
use crate::Repository;
use crate::{ProjectId, ProjectPath, RelativePath};

use crate::test_repository;

#[derive(Default, Debug)]
pub struct FakeRepository {
    global: HashSet<String>,
    by_extension: HashMap<String, Vec<String>>,
    by_project: HashMap<ProjectId, Vec<String>>,
    by_project_and_path: HashMap<(ProjectId, String), Vec<String>>,
    projects: HashMap<String, ProjectId>,
    operations: Vec<String>,
}

impl FakeRepository {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn new_for_tests() -> Result<Self> {
        Ok(Default::default())
    }
}

impl IgnoreStore for FakeRepository {
    fn is_ignored(&self, word: &str) -> Result<bool> {
        Ok(self.global.contains(word))
    }

    fn is_ignored_for_extension(&self, word: &str, extension: &str) -> Result<bool> {
        if let Some(words) = self.by_extension.get(extension) {
            Ok(words.contains(&word.to_string()))
        } else {
            Ok(false)
        }
    }

    fn is_ignored_for_project(&self, word: &str, project_id: ProjectId) -> Result<bool> {
        if let Some(words) = self.by_project.get(&project_id) {
            Ok(words.contains(&word.to_string()))
        } else {
            Ok(false)
        }
    }

    fn is_ignored_for_path(
        &self,
        word: &str,
        project_id: ProjectId,
        path: &RelativePath,
    ) -> Result<bool> {
        if let Some(words) = self
            .by_project_and_path
            .get(&(project_id, path.to_string()))
        {
            Ok(words.contains(&word.to_string()))
        } else {
            Ok(false)
        }
    }

    fn project_exists(&self, project_path: &ProjectPath) -> Result<bool> {
        Ok(self.get_project_id(project_path).is_ok())
    }

    fn new_project(&mut self, project_path: &ProjectPath) -> Result<ProjectId> {
        if self.project_exists(project_path)? {
            bail!("Project in '{}' already exists", project_path);
        }
        let max_id = self.projects.values().max().unwrap_or(&0);
        let new_id = *max_id + 1;

        self.projects.insert(project_path.to_string(), new_id);
        Ok(new_id)
    }

    fn get_project_id(&self, project_path: &ProjectPath) -> Result<ProjectId> {
        let res = self
            .projects
            .get(&project_path.to_string())
            .ok_or_else(|| anyhow!("Could not get project ID for {}", project_path))?;
        Ok(*res)
    }

    fn projects(&self) -> Result<Vec<ProjectInfo>> {
        Ok(self
            .projects
            .iter()
            .map(|(p, i)| ProjectInfo::new(*i, p))
            .collect())
    }

    fn remove_project(&mut self, project_id: ProjectId) -> Result<()> {
        self.projects.retain(|_, i| *i != project_id);
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

    fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let entry = &mut self
            .by_extension
            .entry(extension.to_string())
            .or_insert_with(Vec::new);
        entry.push(word.to_string());
        Ok(())
    }

    fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        let entry = &mut self.by_project.entry(project_id).or_insert_with(Vec::new);
        entry.push(word.to_string());
        Ok(())
    }

    fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        path: &RelativePath,
    ) -> Result<()> {
        let entry = &mut self
            .by_project_and_path
            .entry((project_id, path.to_string()))
            .or_insert_with(Vec::new);
        entry.push(word.to_string());
        Ok(())
    }

    fn remove_ignored(&mut self, word: &str) -> Result<()> {
        let was_present = self.global.remove(word);
        ensure!(was_present, "word was not globally ignored");
        Ok(())
    }

    fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let entry = self
            .by_extension
            .get_mut(extension)
            .ok_or_else(|| anyhow!("no such key"))?;
        entry.retain(|w| w != word);
        Ok(())
    }

    fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let entry = self
            .by_project_and_path
            .get_mut(&(project_id, relative_path.to_string()))
            .ok_or_else(|| anyhow!("no such key"))?;
        entry.retain(|w| w != word);
        Ok(())
    }

    fn remove_ignored_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        let entry = self
            .by_project
            .get_mut(&project_id)
            .ok_or_else(|| anyhow!("no such key"))?;
        entry.retain(|w| w != word);
        Ok(())
    }

    fn insert_operation(&mut self, operation: &Operation) -> Result<()> {
        let as_json = serde_json::to_string(operation).expect("failed to serialize operation");
        self.operations.push(as_json);
        Ok(())
    }

    fn pop_last_operation(&mut self) -> Result<Option<Operation>> {
        let as_json = match self.operations.pop() {
            None => return Ok(None),
            Some(s) => s,
        };
        let res: Operation =
            serde_json::from_str(&as_json).expect("failed to deserialize operation");
        Ok(Some(res))
    }
}

impl Repository for FakeRepository {
    fn undo(&mut self) -> Result<()> {
        todo!()
    }

    fn as_ignore_store(&mut self) -> &mut dyn IgnoreStore {
        todo!()
    }

    fn ensure_project(&mut self, project_path: &ProjectPath) -> Result<crate::Project> {
        todo!()
    }
}

test_repository!(FakeRepository);
