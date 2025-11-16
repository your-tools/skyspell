use std::borrow::Cow;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, anyhow};
use ignore::{Walk, WalkBuilder};
use serde::{Deserialize, Serialize};

use crate::{IgnoreStore, SkipFile, global_path};

pub const SKYSPELL_LOCAL_IGNORE: &str = "skyspell-ignore.toml";

#[derive(Debug, Clone)]
pub struct Project {
    path: ProjectPath,
    skip_file: SkipFile,
}

impl Project {
    pub fn new(path: &Path) -> Result<Self> {
        let skip_file = SkipFile::new(path)?;
        let path = ProjectPath::new(path)?;
        Ok(Self { path, skip_file })
    }

    pub fn path(&self) -> &ProjectPath {
        &self.path
    }

    pub fn as_str(&self) -> Cow<'_, str> {
        self.path.as_str()
    }

    pub fn as_relative_path(&self, path: &str) -> Result<RelativePath> {
        RelativePath::new(self.path(), Path::new(path))
    }

    pub fn get_relative_path(&self, path: &Path) -> Result<RelativePath> {
        RelativePath::new(self.path(), path)
    }

    pub fn ignore_path(&self) -> PathBuf {
        let path = self.path().as_ref();
        path.join(SKYSPELL_LOCAL_IGNORE)
    }

    pub fn ignore_store(&self) -> Result<IgnoreStore> {
        let local_path = self.path.0.join(SKYSPELL_LOCAL_IGNORE);
        let global_path = global_path()?;

        IgnoreStore::load(global_path, local_path)
    }

    pub fn skip_file(&self) -> &SkipFile {
        &self.skip_file
    }

    pub fn walk(&self) -> Result<Walk> {
        Ok(WalkBuilder::new(self.path().as_ref()).build())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectPath(PathBuf);

impl ProjectPath {
    pub fn new(project_path: &Path) -> Result<Self> {
        let path = std::path::absolute(project_path).with_context(|| {
            anyhow!(
                "Could not make project path: {} absolute",
                project_path.display()
            )
        })?;
        Ok(ProjectPath(path))
    }

    pub fn as_str(&self) -> Cow<'_, str> {
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
        let source_path = std::path::absolute(source_path).with_context(|| {
            anyhow!(
                "Could not make relative path absolute: {}",
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

    /// Returns a relative path without checking that
    ///  - it's relative to an existing project
    ///  - it exists
    pub fn from_path_unchecked(path: PathBuf) -> Self {
        Self(path)
    }

    pub fn normalize(&self) -> String {
        self.0.to_string_lossy().replace("\\", "/")
    }

    pub fn file_name(&self) -> Option<Cow<'_, str>> {
        self.0.file_name().map(|x| x.to_string_lossy())
    }

    pub fn extension(&self) -> Option<Cow<'_, str>> {
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
