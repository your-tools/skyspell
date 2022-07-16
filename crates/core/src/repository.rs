use crate::IgnoreStore;
use crate::{Project, ProjectPath};
use anyhow::Result;

pub trait Repository {
    fn undo(&mut self) -> Result<()>;
    fn as_ignore_store(&mut self) -> &mut dyn IgnoreStore;
    fn ensure_project(&mut self, project_path: &ProjectPath) -> Result<Project>;
    fn clean(&mut self) -> Result<()> {
        todo!()
    }
}
