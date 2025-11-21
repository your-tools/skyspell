use std::fmt::Display;
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

    pub fn as_relative_path(&self, path: &str) -> Result<RelativePath> {
        RelativePath::new(&self.path, Path::new(path))
    }

    pub fn get_relative_path(&self, path: &Path) -> Result<RelativePath> {
        RelativePath::new(&self.path, path)
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
pub struct RelativePath(PathBuf);

impl RelativePath {
    pub fn new(project_path: &Path, source_path: &Path) -> Result<Self> {
        let source_path = std::path::absolute(source_path).with_context(|| {
            anyhow!(
                "Could not make relative path absolute: {}",
                source_path.display()
            )
        })?;
        let path = pathdiff::diff_paths(&source_path, project_path).ok_or_else(|| {
            anyhow!(
                "Could not diff paths '{}' and '{}'",
                source_path.display(),
                project_path.display(),
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

    pub fn extension(&self) -> Option<String> {
        self.0.extension().map(|x| x.to_string_lossy().into_owned())
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
