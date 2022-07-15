use std::borrow::Cow;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

pub type ProjectId = i32;

pub const SKYSPELL_IGNORE_FILE: &str = ".skyspell-ignore";
pub const SKYSPELL_CONFIG_FILE: &str = "skyspell.kdl";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Project {
    path: ProjectPath,
    id: ProjectId,
}

impl Project {
    pub fn new(id: ProjectId, path: ProjectPath) -> Self {
        Self { id, path }
    }

    pub fn path(&self) -> &ProjectPath {
        &self.path
    }

    pub fn as_relative_path(&self, path: &str) -> Result<RelativePath> {
        RelativePath::new(self.path(), Path::new(path))
    }

    pub fn id(&self) -> ProjectId {
        self.id
    }

    pub fn ignore_path(&self) -> PathBuf {
        let path = self.path().as_ref();
        path.join(SKYSPELL_IGNORE_FILE)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPath(PathBuf);

impl ProjectPath {
    pub fn new(project_path: &Path) -> Result<Self> {
        let path = std::fs::canonicalize(project_path).with_context(|| {
            anyhow!(
                "Could not canonicalize project path: {}",
                project_path.display()
            )
        })?;
        Ok(ProjectPath(path))
    }

    pub fn as_str(&self) -> Cow<str> {
        self.0.to_string_lossy()
    }
}

impl AsRef<Path> for ProjectPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Display for ProjectPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelativePath(PathBuf);

impl RelativePath {
    pub fn new(project_path: &ProjectPath, source_path: &Path) -> Result<Self> {
        let source_path = std::fs::canonicalize(source_path).with_context(|| {
            anyhow!(
                "Could not canonicalize relative path: {}",
                source_path.display()
            )
        })?;
        let path = pathdiff::diff_paths(&source_path, project_path.as_ref()).ok_or_else(|| {
            anyhow!(
                "Could not diff paths '{}' and '{}'",
                source_path.display(),
                project_path,
            )
        })?;
        Ok(Self(path))
    }

    pub fn as_str(&self) -> Cow<str> {
        self.0.to_string_lossy()
    }

    pub fn file_name(&self) -> Option<Cow<str>> {
        self.0.file_name().map(|x| x.to_string_lossy())
    }

    pub fn extension(&self) -> Option<Cow<str>> {
        self.0.extension().map(|x| x.to_string_lossy())
    }
}

impl AsRef<Path> for RelativePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl Display for RelativePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}
