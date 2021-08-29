use anyhow::{anyhow, bail, Result};

use std::collections::{HashMap, HashSet};
use std::path::Path;

use crate::repository::{ProjectId, ProjectInfo};
use crate::Repository;
use crate::{Project, RelativePath};

#[derive(Default, Debug)]
pub(crate) struct FakeRepository {
    global: HashSet<String>,
    by_extension: HashMap<String, Vec<String>>,
    by_project: HashMap<ProjectId, Vec<String>>,
    by_project_and_path: HashMap<(String, String), Vec<String>>,
    projects: HashMap<String, ProjectId>,
    skip_file_names: HashSet<String>,
    skipped_paths: HashSet<(String, String)>,
}

impl FakeRepository {
    pub(crate) fn new() -> Self {
        Default::default()
    }
}

impl Repository for FakeRepository {
    fn project_exists(&self, project: &Project) -> Result<bool> {
        Ok(self.get_project_id(project).is_ok())
    }

    fn new_project(&mut self, project: &Project) -> Result<ProjectId> {
        if self.project_exists(project)? {
            bail!("Project in '{}' already exists", project);
        }
        let max_id = self.projects.values().max().unwrap_or(&0);
        let new_id = *max_id + 1;

        self.projects.insert(project.to_string(), new_id);
        Ok(new_id)
    }

    fn get_project_id(&self, project: &Project) -> Result<ProjectId> {
        let res = self
            .projects
            .get(&project.to_string())
            .ok_or_else(|| anyhow!("Could not get project ID for {}, project"))?;
        Ok(*res)
    }

    fn projects(&self) -> Result<Vec<ProjectInfo>> {
        Ok(self
            .projects
            .iter()
            .map(|(p, i)| ProjectInfo::new(*i, &p.to_string()))
            .collect())
    }

    fn remove_project(&mut self, path: &std::path::Path) -> Result<()> {
        self.projects.retain(|p, _| Path::new(p) != path);
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

    fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        let entry = &mut self.by_project.entry(project_id).or_insert_with(Vec::new);
        entry.push(word.to_string());
        Ok(())
    }

    fn is_ignored_for_project(&self, word: &str, project_id: ProjectId) -> Result<bool> {
        if let Some(words) = self.by_project.get(&project_id) {
            Ok(words.contains(&word.to_string()))
        } else {
            Ok(false)
        }
    }

    fn ignore_for_path(
        &mut self,
        word: &str,
        project: &Project,
        path: &RelativePath,
    ) -> Result<()> {
        let entry = &mut self
            .by_project_and_path
            .entry((project.to_string(), path.to_string()))
            .or_insert_with(Vec::new);
        entry.push(word.to_string());
        Ok(())
    }

    fn is_ignored_for_path(
        &self,
        word: &str,
        project: &Project,
        path: &RelativePath,
    ) -> Result<bool> {
        if let Some(words) = self
            .by_project_and_path
            .get(&(project.to_string(), path.to_string()))
        {
            Ok(words.contains(&word.to_string()))
        } else {
            Ok(false)
        }
    }

    fn skip_path(&mut self, project: &Project, path: &RelativePath) -> Result<()> {
        self.skipped_paths
            .insert((project.to_string(), path.to_string()));
        Ok(())
    }

    fn is_skipped_path(&self, project: &Project, path: &RelativePath) -> Result<bool> {
        Ok(self
            .skipped_paths
            .contains(&(project.to_string(), path.to_string())))
    }

    fn remove_ignored(&mut self, word: &str) -> Result<()> {
        self.global.remove(word);
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
        project: &Project,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let entry = self
            .by_project_and_path
            .get_mut(&(project.to_string(), relative_path.to_string()))
            .ok_or_else(|| anyhow!("no such key"))?;
        entry.retain(|w| w != word);
        Ok(())
    }

    fn remove_ignored_for_project(&mut self, word: &str, project: &Project) -> Result<()> {
        let project_id = self
            .projects
            .get(&project.to_string())
            .ok_or_else(|| anyhow!("No such project"))?;
        let entry = self
            .by_project
            .get_mut(project_id)
            .ok_or_else(|| anyhow!("no such key"))?;
        entry.retain(|w| w != word);
        Ok(())
    }

    fn unskip_file_name(&mut self, file_name: &str) -> Result<()> {
        self.skip_file_names.retain(|x| x != file_name);
        Ok(())
    }

    fn unskip_path(&mut self, project: &Project, relative_path: &RelativePath) -> Result<()> {
        self.skipped_paths
            .retain(|x| x != &(project.to_string(), relative_path.to_string()));
        Ok(())
    }
}
