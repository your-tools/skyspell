use std::path::Path;

use anyhow::{anyhow, Context, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use directories_next::ProjectDirs;

use crate::repository::{ProjectId, ProjectInfo, Repository};
use crate::sql::models::*;
use crate::sql::schema::*;
use crate::{Project, RelativePath};

diesel_migrations::embed_migrations!("migrations");

pub struct SQLRepository {
    connection: SqliteConnection,
}

pub fn get_default_db_path(lang: &str) -> Result<String> {
    if let Ok(from_env) = std::env::var("SKYSPELL_DB_PATH") {
        return Ok(from_env);
    }

    let project_dirs = ProjectDirs::from("info", "dmerej", "skyspell").ok_or_else(|| {
        anyhow!("Need a home directory to get application directories for skyspell")
    })?;
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Could not create {}", data_dir.display()))?;

    let db_path = data_dir.join(format!("{}.db", lang));
    let url = db_path
        .to_str()
        .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", db_path.display()))?;
    Ok(url.to_string())
}

impl SQLRepository {
    pub fn new(url: &str) -> Result<Self> {
        let connection = SqliteConnection::establish(url)
            .with_context(|| format!("Could not connect to {}", url))?;
        embedded_migrations::run(&connection).with_context(|| "Could not migrate db")?;
        Ok(Self { connection })
    }
    fn get_project_id(&self, project: &Project) -> Result<ProjectId> {
        let res = projects::table
            .filter(projects::path.eq(project.as_str()))
            .select(projects::id)
            .first::<i32>(&self.connection)
            .with_context(|| {
                format!(
                    "Could not get project ID for project '{}'",
                    project.as_str()
                )
            })?;
        Ok(res)
    }
}

impl Repository for SQLRepository {
    fn new_project(&mut self, project: &Project) -> Result<ProjectInfo> {
        let new_project = NewProject {
            path: &project.as_str(),
        };
        diesel::insert_into(projects::table)
            .values(new_project)
            .execute(&self.connection)
            .with_context(|| format!("Could not insert project '{}'", project.as_str()))?;
        let id = self.get_project_id(project)?;
        Ok(ProjectInfo::new(id, &project.to_string()))
    }

    fn get_project_info(&self, project: &Project) -> Result<ProjectInfo> {
        let id = self.get_project_id(project)?;
        Ok(ProjectInfo::new(id, &project.to_string()))
    }

    fn project_exists(&self, project: &Project) -> Result<bool> {
        Ok(projects::table
            .filter(projects::path.eq(project.as_str()))
            .select(projects::id)
            .first::<i32>(&self.connection)
            .optional()
            .with_context(|| format!("Error when looking for project {}", project.as_str()))?
            .is_some())
    }

    fn projects(&self) -> Result<Vec<ProjectInfo>> {
        let rows: Vec<ProjectModel> = projects::table
            .load(&self.connection)
            .with_context(|| "Could not retrieve project list")?;
        Ok(rows
            .iter()
            .map(|x| ProjectInfo::new(x.id, &x.path))
            .collect())
    }

    fn remove_project(&mut self, path: &Path) -> Result<()> {
        diesel::delete(projects::table)
            .filter(projects::path.eq(path.to_string_lossy()))
            .execute(&self.connection)
            .with_context(|| format!("Error when removing project from db {}", path.display()))?;
        Ok(())
    }

    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()> {
        let new_ignored_words: Vec<_> = words.iter().map(|x| NewIgnored { word: x }).collect();
        diesel::insert_or_ignore_into(ignored::table)
            .values(new_ignored_words)
            .execute(&self.connection)
            .with_context(|| "Could not insert ignored words")?;
        Ok(())
    }

    fn ignore(&mut self, word: &str) -> Result<()> {
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored::table)
            .values(NewIgnored { word })
            .execute(&self.connection)
            .with_context(|| "Could not insert ignored word")?;
        Ok(())
    }

    fn is_ignored(&self, word: &str) -> Result<bool> {
        let word = word.to_lowercase();
        Ok(ignored::table
            .filter(ignored::word.eq(word))
            .select(ignored::id)
            .first::<i32>(&self.connection)
            .optional()
            .with_context(|| "Error when checking if word is ignored")?
            .is_some())
    }

    fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored_for_extension::table)
            .values(NewIgnoredForExtension { word, extension })
            .execute(&self.connection)
            .with_context(|| "Could not insert ignored word for extension")?;
        Ok(())
    }

    fn is_ignored_for_extension(&self, word: &str, extension: &str) -> Result<bool> {
        let word = &word.to_lowercase();
        Ok(ignored_for_extension::table
            .filter(ignored_for_extension::word.eq(word))
            .filter(ignored_for_extension::extension.eq(extension))
            .select(ignored_for_extension::id)
            .first::<i32>(&self.connection)
            .optional()
            .with_context(|| "Error when checking if word is ignored for extension")?
            .is_some())
    }

    fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored_for_project::table)
            .values(NewIgnoredForProject { word, project_id })
            .execute(&self.connection)
            .with_context(|| "Could not insert ignored word for project")?;
        Ok(())
    }

    fn is_ignored_for_project(&self, word: &str, project: &Project) -> Result<bool> {
        let project_id = self.get_project_id(project)?;
        let word = &word.to_lowercase();
        Ok(ignored_for_project::table
            .filter(ignored_for_project::project_id.eq(project_id))
            .filter(ignored_for_project::word.eq(word))
            .select(ignored_for_project::id)
            .first::<i32>(&self.connection)
            .optional()
            .with_context(|| "Error when checking if word is ignored for project")?
            .is_some())
    }

    fn ignore_for_path(
        &mut self,
        word: &str,
        project: &Project,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let word = &word.to_lowercase();
        let project_id = self.get_project_id(project)?;
        diesel::insert_or_ignore_into(ignored_for_path::table)
            .values(NewIgnoredForPath {
                word,
                project_id,
                path: &relative_path.as_str(),
            })
            .execute(&self.connection)
            .with_context(|| "Could not insert ignored word for path")?;
        Ok(())
    }

    fn is_ignored_for_path(
        &self,
        word: &str,
        project: &Project,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        let word = &word.to_lowercase();
        let project_id = self.get_project_id(project)?;
        Ok(ignored_for_path::table
            .filter(ignored_for_path::project_id.eq(project_id))
            .filter(ignored_for_path::word.eq(word))
            .filter(ignored_for_path::path.eq(relative_path.as_str()))
            .select(ignored_for_path::id)
            .first::<i32>(&self.connection)
            .optional()
            .with_context(|| "Error when checking if word is ignored for given path")?
            .is_some())
    }

    fn skip_file_name(&mut self, file_name: &str) -> Result<()> {
        diesel::insert_or_ignore_into(skipped_file_names::table)
            .values(NewSkippedFileName { file_name })
            .execute(&self.connection)
            .with_context(|| "Could not insert file name to the list of skipped file names")?;
        Ok(())
    }

    fn is_skipped_file_name(&self, file_name: &str) -> Result<bool> {
        Ok(skipped_file_names::table
            .filter(skipped_file_names::file_name.eq(file_name))
            .select(skipped_file_names::id)
            .first::<i32>(&self.connection)
            .optional()
            .with_context(|| "Error when checking if file name should be skipped")?
            .is_some())
    }

    fn skip_path(&mut self, project: &Project, relative_path: &RelativePath) -> Result<()> {
        let project_id = self.get_project_id(project)?;
        diesel::insert_or_ignore_into(skipped_paths::table)
            .values(NewSkippedPath {
                path: &relative_path.as_str(),
                project_id,
            })
            .execute(&self.connection)
            .with_context(|| "Could not insert file path to the list of skipped file paths")?;
        Ok(())
    }

    fn is_skipped_path(&self, project: &Project, relative_path: &RelativePath) -> Result<bool> {
        let project_id = self.get_project_id(project)?;
        Ok(skipped_paths::table
            .filter(skipped_paths::project_id.eq(project_id))
            .filter(skipped_paths::path.eq(relative_path.as_str()))
            .select(skipped_paths::id)
            .first::<i32>(&self.connection)
            .optional()
            .with_context(|| "Error when checking if path is skipped")?
            .is_some())
    }

    fn remove_ignored(&mut self, word: &str) -> Result<()> {
        let word = word.to_lowercase();
        diesel::delete(ignored::table)
            .filter(ignored::word.eq(word))
            .execute(&self.connection)
            .with_context(|| "Could not remove word from global ignored list")?;
        Ok(())
    }

    fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let word = word.to_lowercase();
        diesel::delete(ignored_for_extension::table)
            .filter(ignored_for_extension::extension.eq(extension))
            .filter(ignored_for_extension::word.eq(word))
            .execute(&self.connection)
            .with_context(|| "Could not remove word from ignore list for extension")?;
        Ok(())
    }

    fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project: &Project,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let word = word.to_lowercase();
        let project_id = self.get_project_id(project)?;
        diesel::delete(ignored_for_path::table)
            .filter(ignored_for_path::word.eq(word))
            .filter(ignored_for_path::project_id.eq(project_id))
            .filter(ignored_for_path::path.eq(relative_path.as_str()))
            .execute(&self.connection)
            .with_context(|| "Could not remove word from ignore list for path")?;
        Ok(())
    }

    fn remove_ignored_for_project(&mut self, word: &str, project: &Project) -> Result<()> {
        let word = word.to_lowercase();
        let project_id = self.get_project_id(project)?;
        diesel::delete(ignored_for_project::table)
            .filter(ignored_for_project::word.eq(word))
            .filter(ignored_for_project::project_id.eq(project_id))
            .execute(&self.connection)
            .with_context(|| "Could not remove word from ignore list for project")?;
        Ok(())
    }

    fn unskip_file_name(&mut self, file_name: &str) -> Result<()> {
        diesel::delete(skipped_file_names::table)
            .filter(skipped_file_names::file_name.eq(file_name))
            .execute(&self.connection)
            .with_context(|| "Could not remove file name from skip list")?;
        Ok(())
    }

    fn unskip_path(&mut self, project: &Project, relative_path: &RelativePath) -> Result<()> {
        let project_id = self.get_project_id(project)?;
        diesel::delete(skipped_paths::table)
            .filter(skipped_paths::path.eq(relative_path.as_str()))
            .filter(skipped_paths::project_id.eq(project_id))
            .execute(&self.connection)
            .with_context(|| "Could not remove file path from skip list")?;
        Ok(())
    }
}