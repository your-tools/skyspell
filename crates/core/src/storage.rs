use anyhow::{bail, Result};

use crate::{IgnoreStore, Repository};
use crate::{Project, ProjectPath};

/// We have two backends to store ignore words One can manipulate ignored words,
/// but the other is more powerful because it can store and retriev operations

/// Thus, we crate an enum to represent the "capabilities" of a storage - either
/// it implements Repository with all its methods, or it implements IgnoreStore
/// with a subset of these.
pub enum StorageBackend {
    IgnoreStore(Box<dyn IgnoreStore>),
    Repository(Box<dyn Repository>),
}

impl StorageBackend {
    pub fn as_ignore_store(&mut self) -> &mut dyn IgnoreStore {
        match self {
            StorageBackend::IgnoreStore(i) => i.as_mut(),
            StorageBackend::Repository(r) => r.as_ignore_store(),
        }
    }

    pub fn as_repository(&mut self) -> Option<&mut dyn Repository> {
        match self {
            StorageBackend::IgnoreStore(_) => None,
            StorageBackend::Repository(r) => Some(r.as_mut()),
        }
    }

    pub fn ensure_project(&mut self, project_path: &ProjectPath) -> Result<Project> {
        let project_id = match self.as_repository() {
            Some(r) => {
                r.new_project(project_path)?;
                r.get_project_id(project_path)?
            }
            None => 42,
        };

        Ok(Project::new(project_id, project_path.clone()))
    }

    fn clean(&mut self) -> Result<()> {
        todo!()
    }

    pub fn undo(&mut self) -> Result<()> {
        match self {
            StorageBackend::IgnoreStore(_) => bail!("Cannot undo"),
            StorageBackend::Repository(r) => r.undo(),
        }
    }
}
