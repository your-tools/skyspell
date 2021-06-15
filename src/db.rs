use std::fmt::{Debug, Formatter};

use anyhow::{anyhow, Context, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use platform_dirs::AppDirs;

use crate::models::*;
use crate::path::{ProjectPath, RelativePath};
use crate::repository::Repository;
use crate::schema::*;

diesel_migrations::embed_migrations!("migrations");

pub(crate) struct Db {
    connection: SqliteConnection,
    url: String,
}

impl Debug for Db {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "Db<{}>", self.url)
    }
}

impl Db {
    pub(crate) fn connect(url: &str) -> Result<Self> {
        let connection = SqliteConnection::establish(&url)
            .with_context(|| format!("Could not connect to {}", url))?;
        embedded_migrations::run(&connection).with_context(|| "Could not migrate db")?;
        Ok(Self {
            connection,
            url: url.to_owned(),
        })
    }

    pub(crate) fn open(lang: &str) -> Result<Self> {
        let app_dirs = AppDirs::new(Some("skyspell"), false)
            .with_context(|| "Could not get app dirs for skyspell application")?;
        let data_dir = app_dirs.data_dir;
        std::fs::create_dir_all(&data_dir)
            .with_context(|| format!("Could not create {}", data_dir.display()))?;

        let db_path = &data_dir.join(format!("{}.db", lang));
        let db_path = db_path
            .to_str()
            .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", db_path.display()))?;
        Self::connect(db_path)
    }

    pub(crate) fn remove_ignored(&mut self, word: &str) -> Result<()> {
        let word = word.to_lowercase();
        diesel::delete(ignored::table)
            .filter(ignored::word.eq(word))
            .execute(&self.connection)?;
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
            .execute(&self.connection)?;
        Ok(())
    }

    pub(crate) fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project_path: &ProjectPath,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let word = word.to_lowercase();
        let project_id = self.get_project_id(project_path)?;
        diesel::delete(ignored_for_path::table)
            .filter(ignored_for_path::word.eq(word))
            .filter(ignored_for_path::project_id.eq(project_id))
            .filter(ignored_for_path::path.eq(relative_path.as_str()))
            .execute(&self.connection)?;
        Ok(())
    }

    pub(crate) fn remove_ignored_for_project(
        &mut self,
        word: &str,
        project_path: &ProjectPath,
    ) -> Result<()> {
        let word = word.to_lowercase();
        let project_id = self.get_project_id(project_path)?;
        diesel::delete(ignored_for_project::table)
            .filter(ignored_for_project::word.eq(word))
            .filter(ignored_for_project::project_id.eq(project_id))
            .execute(&self.connection)?;
        Ok(())
    }

    pub(crate) fn unskip_file_name(&mut self, file_name: &str) -> Result<()> {
        diesel::delete(skipped_file_names::table)
            .filter(skipped_file_names::file_name.eq(file_name))
            .execute(&self.connection)?;
        Ok(())
    }

    pub(crate) fn unskip_path(
        &mut self,
        project_path: &ProjectPath,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let project_id = self.get_project_id(project_path)?;
        diesel::delete(skipped_paths::table)
            .filter(skipped_paths::path.eq(relative_path.as_str()))
            .filter(skipped_paths::project_id.eq(project_id))
            .execute(&self.connection)?;
        Ok(())
    }

    fn get_project_id(&self, project_path: &ProjectPath) -> Result<i32> {
        let res = projects::table
            .filter(projects::path.eq(project_path.as_str()))
            .select(projects::id)
            .first::<i32>(&self.connection)?;
        Ok(res)
    }
}

impl Repository for Db {
    fn new_project(&mut self, path: &ProjectPath) -> Result<()> {
        let new_project = NewProject {
            path: &path.as_str(),
        };
        diesel::insert_into(projects::table)
            .values(new_project)
            .execute(&self.connection)?;
        Ok(())
    }

    fn project_exists(&self, path: &ProjectPath) -> Result<bool> {
        Ok(projects::table
            .filter(projects::path.eq(path.as_str()))
            .select(projects::id)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()> {
        let new_ignored_words: Vec<_> = words.iter().map(|x| NewIgnored { word: x }).collect();
        diesel::insert_or_ignore_into(ignored::table)
            .values(new_ignored_words)
            .execute(&self.connection)?;
        Ok(())
    }

    fn ignore(&mut self, word: &str) -> Result<()> {
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored::table)
            .values(NewIgnored { word })
            .execute(&self.connection)?;
        Ok(())
    }

    fn is_ignored(&self, word: &str) -> Result<bool> {
        let word = word.to_lowercase();
        Ok(ignored::table
            .filter(ignored::word.eq(word))
            .select(ignored::id)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }

    fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored_for_extension::table)
            .values(NewIgnoredForExtension { word, extension })
            .execute(&self.connection)?;
        Ok(())
    }

    fn is_ignored_for_extension(&self, word: &str, extension: &str) -> Result<bool> {
        let word = &word.to_lowercase();
        Ok(ignored_for_extension::table
            .filter(ignored_for_extension::word.eq(word))
            .filter(ignored_for_extension::extension.eq(extension))
            .select(ignored_for_extension::id)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }

    fn ignore_for_project(&mut self, word: &str, project_path: &ProjectPath) -> Result<()> {
        let project_id = self.get_project_id(project_path)?;
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored_for_project::table)
            .values(NewIgnoredForProject { word, project_id })
            .execute(&self.connection)?;
        Ok(())
    }

    fn is_ignored_for_project(&self, word: &str, project_path: &ProjectPath) -> Result<bool> {
        let project_id = self.get_project_id(project_path)?;
        let word = &word.to_lowercase();
        Ok(ignored_for_project::table
            .filter(ignored_for_project::project_id.eq(project_id))
            .filter(ignored_for_project::word.eq(word))
            .select(ignored_for_project::id)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }

    fn ignore_for_path(
        &mut self,
        word: &str,
        project_path: &ProjectPath,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let word = &word.to_lowercase();
        let project_id = self.get_project_id(project_path)?;
        diesel::insert_or_ignore_into(ignored_for_path::table)
            .values(NewIgnoredForPath {
                word,
                project_id,
                path: &relative_path.as_str(),
            })
            .execute(&self.connection)?;
        Ok(())
    }

    fn is_ignored_for_path(
        &self,
        word: &str,
        project_path: &ProjectPath,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        let word = &word.to_lowercase();
        let project_id = self.get_project_id(project_path)?;
        Ok(ignored_for_path::table
            .filter(ignored_for_path::project_id.eq(project_id))
            .filter(ignored_for_path::word.eq(word))
            .filter(ignored_for_path::path.eq(relative_path.as_str()))
            .select(ignored_for_path::id)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }

    fn skip_file_name(&mut self, file_name: &str) -> Result<()> {
        diesel::insert_or_ignore_into(skipped_file_names::table)
            .values(NewSkippedFileName { file_name })
            .execute(&self.connection)?;
        Ok(())
    }

    fn is_skipped_file_name(&self, file_name: &str) -> Result<bool> {
        Ok(skipped_file_names::table
            .filter(skipped_file_names::file_name.eq(file_name))
            .select(skipped_file_names::id)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }

    fn skip_path(
        &mut self,
        project_path: &ProjectPath,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let project_id = self.get_project_id(project_path)?;
        diesel::insert_or_ignore_into(skipped_paths::table)
            .values(NewSkippedPath {
                path: &relative_path.as_str(),
                project_id,
            })
            .execute(&self.connection)?;
        Ok(())
    }

    fn is_skipped_path(
        &self,
        project_path: &ProjectPath,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        let project_id = self.get_project_id(project_path)?;
        Ok(skipped_paths::table
            .filter(skipped_paths::project_id.eq(project_id))
            .filter(skipped_paths::path.eq(relative_path.as_str()))
            .select(skipped_paths::id)
            .first::<i32>(&self.connection)
            .optional()?
            .is_some())
    }
}
