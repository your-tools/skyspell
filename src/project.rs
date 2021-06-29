use anyhow::{anyhow, Context, Result};
use std::borrow::Cow;
use std::fmt::Display;
use std::path::{Path, PathBuf};

// Note: we store the paths in the DB using lossy string representation because it's really
// convenient, although technically not correct
//
// An other option would be to store the OsStr representation as binary in the DB

pub(crate) struct Project(PathBuf);

impl Project {
    pub(crate) fn new(project_path: &Path) -> Result<Self> {
        let path = std::fs::canonicalize(project_path).with_context(|| {
            anyhow!(
                "Could not canonicalize project path: {}",
                project_path.display()
            )
        })?;
        Ok(Project(path))
    }

    pub(crate) fn as_str(&self) -> Cow<str> {
        self.0.to_string_lossy()
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }
}

impl Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.display())
    }
}

#[derive(Debug)]
pub(crate) struct RelativePath(PathBuf);

impl RelativePath {
    pub(crate) fn new(project: &Project, source_path: &Path) -> Result<Self> {
        let source_path = std::fs::canonicalize(source_path).with_context(|| {
            anyhow!(
                "Could not canonicalize relative path: {}",
                source_path.display()
            )
        })?;
        let path = pathdiff::diff_paths(&source_path, project.path()).ok_or_else(|| {
            anyhow!(
                "Could not diff paths '{}' and '{}'",
                source_path.display(),
                project,
            )
        })?;
        Ok(Self(path))
    }

    pub(crate) fn as_str(&self) -> Cow<str> {
        self.0.to_string_lossy()
    }

    pub(crate) fn file_name(&self) -> Option<Cow<str>> {
        self.0.file_name().map(|x| x.to_string_lossy())
    }

    pub(crate) fn extension(&self) -> Option<Cow<str>> {
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
