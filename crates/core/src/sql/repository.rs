// Note: we store the paths in the DB using lossy string representation
// because it's really convenient, although technically not correct
//
// An other option would be to store the OsStr representation as binary
// in the DB

use anyhow::{anyhow, ensure, Context, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use directories_next::ProjectDirs;

use crate::sql::models::*;
use crate::sql::schema::*;
use crate::{IgnoreStore, Repository};
use crate::{Operation, ProjectInfo};
use crate::{ProjectId, ProjectPath, RelativePath};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub fn get_default_db_path(lang: &str) -> Result<String> {
    let project_dirs = ProjectDirs::from("info", "dmerej", "skyspell").ok_or_else(|| {
        anyhow!("Need a home directory to get application directories for skyspell")
    })?;
    let data_dir = project_dirs.data_dir();
    std::fs::create_dir_all(data_dir)
        .with_context(|| format!("Could not create {}", data_dir.display()))?;

    let db_path = data_dir.join(format!("{}.db", lang));
    let url = db_path
        .to_str()
        .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", db_path.display()))?;
    Ok(url.to_string())
}

pub struct SQLRepository {
    pub connection: SqliteConnection,
}

impl SQLRepository {
    pub fn new(url: &str) -> Result<Self> {
        let mut connection = SqliteConnection::establish(url)
            .with_context(|| format!("Could not connect to {}", url))?;
        let outcome = connection.run_pending_migrations(MIGRATIONS);
        outcome.map_err(|e| anyhow!("Could not migrate db: {e}"))?;
        Ok(Self { connection })
    }

    pub fn new_for_tests() -> Result<Self> {
        Self::new(":memory:")
    }
}

impl IgnoreStore for SQLRepository {
    fn is_ignored(&mut self, word: &str) -> Result<bool> {
        let word = word.to_lowercase();
        Ok(ignored::table
            .filter(ignored::word.eq(word))
            .select(ignored::id)
            .first::<i32>(&mut self.connection)
            .optional()
            .with_context(|| "Error when checking if word is ignored")?
            .is_some())
    }

    fn is_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<bool> {
        let word = &word.to_lowercase();
        Ok(ignored_for_extension::table
            .filter(ignored_for_extension::word.eq(word))
            .filter(ignored_for_extension::extension.eq(extension))
            .select(ignored_for_extension::id)
            .first::<i32>(&mut self.connection)
            .optional()
            .with_context(|| "Error when checking if word is ignored for extension")?
            .is_some())
    }

    fn is_ignored_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<bool> {
        let word = &word.to_lowercase();
        Ok(ignored_for_project::table
            .filter(ignored_for_project::project_id.eq(project_id))
            .filter(ignored_for_project::word.eq(word))
            .select(ignored_for_project::id)
            .first::<i32>(&mut self.connection)
            .optional()
            .with_context(|| "Error when checking if word is ignored for project")?
            .is_some())
    }

    fn is_ignored_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        let word = &word.to_lowercase();
        Ok(ignored_for_path::table
            .filter(ignored_for_path::project_id.eq(project_id))
            .filter(ignored_for_path::word.eq(word))
            .filter(ignored_for_path::path.eq(relative_path.as_str()))
            .select(ignored_for_path::id)
            .first::<i32>(&mut self.connection)
            .optional()
            .with_context(|| "Error when checking if word is ignored for given path")?
            .is_some())
    }

    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()> {
        let new_ignored_words: Vec<_> = words.iter().map(|x| NewIgnored { word: x }).collect();
        diesel::insert_or_ignore_into(ignored::table)
            .values(new_ignored_words)
            .execute(&mut self.connection)
            .with_context(|| "Could not insert ignored words")?;
        Ok(())
    }

    fn ignore(&mut self, word: &str) -> Result<()> {
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored::table)
            .values(NewIgnored { word })
            .execute(&mut self.connection)
            .with_context(|| "Could not insert ignored word")?;
        Ok(())
    }

    fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored_for_extension::table)
            .values(NewIgnoredForExtension { word, extension })
            .execute(&mut self.connection)
            .with_context(|| "Could not insert ignored word for extension")?;
        Ok(())
    }

    fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored_for_project::table)
            .values(NewIgnoredForProject { word, project_id })
            .execute(&mut self.connection)
            .with_context(|| "Could not insert ignored word for project")?;
        Ok(())
    }

    fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let word = &word.to_lowercase();
        diesel::insert_or_ignore_into(ignored_for_path::table)
            .values(NewIgnoredForPath {
                word,
                project_id,
                path: &relative_path.as_str(),
            })
            .execute(&mut self.connection)
            .with_context(|| "Could not insert ignored word for path")?;
        Ok(())
    }

    fn remove_ignored(&mut self, word: &str) -> Result<()> {
        let word = word.to_lowercase();
        let num_rows = diesel::delete(ignored::table)
            .filter(ignored::word.eq(word))
            .execute(&mut self.connection)
            .with_context(|| "Could not remove word from global ignored list")?;
        ensure!(num_rows != 0, "word was not globally ignored");
        Ok(())
    }

    fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<()> {
        let word = word.to_lowercase();
        let num_rows = diesel::delete(ignored_for_extension::table)
            .filter(ignored_for_extension::extension.eq(extension))
            .filter(ignored_for_extension::word.eq(word))
            .execute(&mut self.connection)
            .with_context(|| "Could not remove word from ignore list for extension")?;
        ensure!(
            num_rows != 0,
            "word was not in the ignore list for the given extension"
        );
        Ok(())
    }

    fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()> {
        let word = word.to_lowercase();
        let num_rows = diesel::delete(ignored_for_path::table)
            .filter(ignored_for_path::word.eq(word))
            .filter(ignored_for_path::project_id.eq(project_id))
            .filter(ignored_for_path::path.eq(relative_path.as_str()))
            .execute(&mut self.connection)
            .with_context(|| "Could not remove word from ignore list for path")?;
        ensure!(
            num_rows != 0,
            "word was not in the ignore list for the given project and path"
        );
        Ok(())
    }

    fn remove_ignored_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()> {
        let word = word.to_lowercase();
        diesel::delete(ignored_for_project::table)
            .filter(ignored_for_project::word.eq(word))
            .filter(ignored_for_project::project_id.eq(project_id))
            .execute(&mut self.connection)
            .with_context(|| "Could not remove word from ignore list for project")?;
        Ok(())
    }
}

impl Repository for SQLRepository {
    fn ignore_store_mut(&mut self) -> &mut dyn IgnoreStore {
        self
    }

    fn ignore_store(&self) -> &dyn IgnoreStore {
        self
    }

    fn new_project(&mut self, project: &ProjectPath) -> Result<ProjectId> {
        let new_project = NewProject {
            path: &project.as_str(),
        };
        diesel::insert_into(projects::table)
            .values(new_project)
            .execute(&mut self.connection)
            .with_context(|| format!("Could not insert project '{}'", project.as_str()))?;
        self.get_project_id(project)
    }

    fn get_project_id(&mut self, project: &ProjectPath) -> Result<ProjectId> {
        let res = projects::table
            .filter(projects::path.eq(project.as_str()))
            .select(projects::id)
            .first::<i32>(&mut self.connection)
            .with_context(|| {
                format!(
                    "Could not get project ID for project '{}'",
                    project.as_str()
                )
            })?;
        Ok(res)
    }

    fn project_exists(&mut self, project: &ProjectPath) -> Result<bool> {
        Ok(projects::table
            .filter(projects::path.eq(project.as_str()))
            .select(projects::id)
            .first::<i32>(&mut self.connection)
            .optional()
            .with_context(|| format!("Error when looking for project {}", project.as_str()))?
            .is_some())
    }

    fn projects(&mut self) -> Result<Vec<ProjectInfo>> {
        let rows: Vec<ProjectModel> = projects::table
            .load(&mut self.connection)
            .with_context(|| "Could not retrieve project list")?;
        Ok(rows
            .iter()
            .map(|x| ProjectInfo::new(x.id, &x.path))
            .collect())
    }

    fn remove_project(&mut self, project_id: ProjectId) -> Result<()> {
        diesel::delete(projects::table)
            .filter(projects::id.eq(project_id))
            .execute(&mut self.connection)
            .with_context(|| format!("Error when removing project #{} from db", project_id))?;
        Ok(())
    }

    fn insert_operation(&mut self, operation: &Operation) -> Result<()> {
        let as_json = serde_json::to_string(operation).expect("Could not deserialize operation");
        let now = time::OffsetDateTime::now_utc();
        let timestamp = now.unix_timestamp();
        let new_operation = NewOperation {
            json: &as_json,
            timestamp,
        };
        diesel::insert_into(operations::table)
            .values(new_operation)
            .execute(&mut self.connection)
            .with_context(|| format!("Could not insert operation '{:?}'", operation))?;
        Ok(())
    }

    fn pop_last_operation(&mut self) -> Result<Option<Operation>> {
        // Note: since we are going to mutate the operations table,
        // we might as well delete old entries, making sure to only
        // keep the most recent values
        let res = operations::table
            .order_by(operations::timestamp.desc())
            .first::<OperationModel>(&mut self.connection)
            .optional()
            .with_context(|| "Could not fetch last operation")?;

        let OperationModel { id, json, .. } = match res {
            None => return Ok(None),
            Some(v) => v,
        };

        diesel::delete(operations::table)
            .filter(operations::id.eq(id))
            .execute(&mut self.connection)
            .with_context(|| "Could not delete last operation")?;

        let oldest_operation = operations::table
            .order_by(operations::timestamp.desc())
            .offset(100)
            .first::<OperationModel>(&mut self.connection)
            .optional()
            .with_context(|| "Could not get date of the oldest operation")?;

        if let Some(o) = oldest_operation {
            diesel::delete(operations::table)
                .filter(operations::timestamp.lt(o.timestamp))
                .execute(&mut self.connection)
                .with_context(|| "Could not delete old operations")?;
        }

        let operation: Operation = serde_json::from_str(&json)
            .with_context(|| "Could not deserialize operation from db")?;
        Ok(Some(operation))
    }
}
