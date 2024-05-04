use crate::{Dictionary, IgnoreStore, Operation};
use crate::{Project, RelativePath};
use anyhow::{anyhow, bail, Context, Result};
use directories_next::BaseDirs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub trait Checker<D: Dictionary> {
    type Context;

    fn dictionary(&self) -> &D;

    fn project(&self) -> &Project;

    fn to_relative_path(&self, path: &Path) -> Result<RelativePath> {
        let project_path = self.project().path();
        RelativePath::new(project_path, path)
    }

    // Were all the errors handled properly?
    fn success(&self) -> Result<()>;

    fn ignore_store(&mut self) -> &mut IgnoreStore;

    fn state(&mut self) -> Option<&mut CheckerState> {
        None
    }

    fn handle_error(
        &mut self,
        error: &str,
        path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()>;

    fn handle_token(
        &mut self,
        token: &str,
        relative_path: &RelativePath,
        context: &Self::Context,
    ) -> Result<()> {
        let dictionary = self.dictionary();
        let in_dict = dictionary.check(token)?;
        if in_dict {
            return Ok(());
        }
        let should_ignore = self.ignore_store().should_ignore(token, relative_path);
        if !should_ignore {
            self.handle_error(token, relative_path, context)?
        }
        Ok(())
    }

    fn apply_operation(&mut self, mut operation: Operation) -> Result<()> {
        let store = self.ignore_store();
        operation.execute(store)?;
        if let Some(state) = self.state() {
            state.set_last_operation(operation.clone())?;
        }
        Ok(())
    }

    fn undo(&mut self) -> Result<()> {
        let state = match self.state() {
            None => bail!("Cannot undo"),
            Some(s) => s,
        };
        let last_operation = state.pop_last_operation()?;
        let mut last_operation = match last_operation {
            None => bail!("Nothing to undo"),
            Some(o) => o,
        };
        let store = self.ignore_store();
        last_operation.undo(store)
    }
}

pub struct CheckerState {
    storage_path: PathBuf,
    inner: StateInner,
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct StateInner {
    last_operation: Option<Operation>,
}

impl CheckerState {
    pub fn load(state_toml: Option<PathBuf>) -> Result<Self> {
        let state_toml = match state_toml {
            None => {
                let base_dirs =
                    BaseDirs::new().ok_or_else(|| anyhow!("Could not get home directory"))?;
                let data_dir = base_dirs.data_dir().join("skyspell");
                std::fs::create_dir_all(&data_dir)
                    .with_context(|| format!("Could not create data dir {}", data_dir.display()))?;
                data_dir.join("state.toml")
            }
            Some(p) => p,
        };
        let inner: StateInner = if state_toml.exists() {
            let contents = std::fs::read_to_string(&state_toml)
                .with_context(|| format!("Could not read from {}", state_toml.display()))?;
            toml_edit::de::from_str(&contents)
                .with_context(|| format!("Could not parse {}", state_toml.display()))?
        } else {
            Default::default()
        };

        Ok(CheckerState {
            storage_path: state_toml,
            inner,
        })
    }

    pub fn set_last_operation(&mut self, operation: Operation) -> Result<()> {
        self.inner.last_operation = Some(operation);
        self.save()
    }

    pub fn pop_last_operation(&mut self) -> Result<Option<Operation>> {
        let result = self.inner.last_operation.take();
        self.save()?;
        Ok(result)
    }

    fn save(&self) -> Result<()> {
        let contents = toml_edit::ser::to_string_pretty(&self.inner)
            .with_context(|| "Could not serialize state")?;
        std::fs::write(&self.storage_path, contents)
            .with_context(|| "Could not write to storage path")?;
        Ok(())
    }
}
