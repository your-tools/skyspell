use anyhow::{anyhow, Context, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use platform_dirs::AppDirs;

use crate::models::*;
use crate::repository::Repository;
use crate::schema::*;
use crate::{Project, RelativePath};

diesel_migrations::embed_migrations!("migrations");

pub(crate) struct SQLRepository {
    connection: SqliteConnection,
}

pub(crate) fn get_default_db_path(lang: &str) -> Result<String> {
    let app_dirs = AppDirs::new(Some("skyspell"), false)
        .with_context(|| "Could not get app dirs for skyspell application")?;
    let data_dir = app_dirs.data_dir;
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Could not create {}", data_dir.display()))?;

    let db_path = data_dir.join(format!("{}.db", lang));
    let url = db_path
        .to_str()
        .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", db_path.display()))?;
    Ok(url.to_string())
}

impl SQLRepository {
    pub(crate) fn new(url: &str) -> Result<Self> {
        let connection = SqliteConnection::establish(&url)
            .with_context(|| format!("Could not connect to {}", url))?;
        embedded_migrations::run(&connection).with_context(|| "Could not migrate db")?;
        Ok(Self { connection })
    }

    pub(crate) fn remove_ignored(&mut self, word: &str) -> Result<()> {
        let word = word.to_lowercase();
        diesel::delete(ignored::table)
            .filter(ignored::word.eq(word))
            .execute(&self.connection)
            .with_context(|| "Could not remove word from global ignored list")?;
        Ok(())
    }

    pub(crate) fn remove_ignored_for_extension(
        &mut self,
        word: &str,
        extension: &str,
    ) -> Result<()> {
        let word = word.to_lowercase();
        diesel::delete(ignored_for_extension::table)
            .filter(ignored_for_extension::extension.eq(extension))
            .filter(ignored_for_extension::word.eq(word))
            .execute(&self.connection)
            .with_context(|| "Could not remove word from ignore list for extension")?;
        Ok(())
    }

    pub(crate) fn remove_ignored_for_path(
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

    pub(crate) fn remove_ignored_for_project(
        &mut self,
        word: &str,
        project: &Project,
    ) -> Result<()> {
        let word = word.to_lowercase();
        let project_id = self.get_project_id(project)?;
        diesel::delete(ignored_for_project::table)
            .filter(ignored_for_project::word.eq(word))
            .filter(ignored_for_project::project_id.eq(project_id))
            .execute(&self.connection)
            .with_context(|| "Could not remove word from ignore list for project")?;
        Ok(())
    }

    pub(crate) fn unskip_file_name(&mut self, file_name: &str) -> Result<()> {
        diesel::delete(skipped_file_names::table)
            .filter(skipped_file_names::file_name.eq(file_name))
            .execute(&self.connection)
            .with_context(|| "Could not remove file name from skip list")?;
        Ok(())
    }

    pub(crate) fn unskip_path(
        &mut self,
        project: &Project,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let project_id = self.get_project_id(project)?;
        diesel::delete(skipped_paths::table)
            .filter(skipped_paths::path.eq(relative_path.as_str()))
            .filter(skipped_paths::project_id.eq(project_id))
            .execute(&self.connection)
            .with_context(|| "Could not remove file path from skip list")?;
        Ok(())
    }

    fn get_project_id(&self, project: &Project) -> Result<i32> {
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
    fn new_project(&mut self, project: &Project) -> Result<()> {
        let new_project = NewProject {
            path: &project.as_str(),
        };
        diesel::insert_into(projects::table)
            .values(new_project)
            .execute(&self.connection)
            .with_context(|| format!("Could not insert project '{}'", project.as_str()))?;
        Ok(())
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

    fn ignore_for_project(&mut self, word: &str, project: &Project) -> Result<()> {
        let project_id = self.get_project_id(project)?;
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
}
