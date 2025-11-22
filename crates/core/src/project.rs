use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use ignore::{Walk, WalkBuilder};
use serde::{Deserialize, Serialize};

use crate::{IgnoreStore, SkipFile, global_path};

pub const SKYSPELL_LOCAL_IGNORE: &str = "skyspell-ignore.toml";

#[derive(Debug, Clone)]
pub struct Project {
    path: PathBuf,
    skip_file: SkipFile,
}

impl Project {
    pub fn new(path: &Path) -> Result<Self> {
        let skip_file = SkipFile::new(path)?;
        let path = std::path::absolute(path)
            .with_context(|| format!("Could not make path {path:?} absolute "))?;
        Ok(Self { path, skip_file })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn path_string(&self) -> String {
        self.path.to_string_lossy().into_owned()
    }

    pub fn new_project_file<P: AsRef<Path>>(&self, path: P) -> Result<ProjectFile> {
        ProjectFile::new(self, path.as_ref())
    }

    pub fn ignore_path(&self) -> PathBuf {
        let path = self.path();
        path.join(SKYSPELL_LOCAL_IGNORE)
    }

    pub fn ignore_store(&self) -> Result<IgnoreStore> {
        let local_path = self.path.join(SKYSPELL_LOCAL_IGNORE);
        let global_path = global_path()?;

        IgnoreStore::load(global_path, local_path)
    }

    pub fn skip_file(&self) -> &SkipFile {
        &self.skip_file
    }

    pub fn walk(&self) -> Result<Walk> {
        Ok(WalkBuilder::new(self.path()).build())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectFile {
    name: String,
    extension: Option<String>,
    full_path: PathBuf,
}

impl ProjectFile {
    pub fn new(project: &Project, source_path: &Path) -> Result<Self> {
        let extension = source_path
            .extension()
            .map(|x| x.to_string_lossy().into_owned());

        let project_path = project.path();
        let full_path = std::path::absolute(source_path).with_context(|| {
            anyhow!(
                "Could not make relative path absolute: {}",
                source_path.display()
            )
        })?;
        let relative_path = pathdiff::diff_paths(&full_path, project_path).ok_or_else(|| {
            anyhow!(
                "Could not diff paths '{}' and '{}'",
                full_path.display(),
                project_path.display(),
            )
        })?;
        let name = relative_path.to_string_lossy().into_owned();
        let name = name.replace("\\", "/");
        Ok(Self {
            name,
            extension,
            full_path,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn extension(&self) -> Option<&str> {
        self.extension.as_deref()
    }

    pub fn full_path(&self) -> &Path {
        &self.full_path
    }
}
