use anyhow::Result;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::{Walk, WalkBuilder};

use crate::project::SKYSPELL_LOCAL_IGNORE;
use crate::LocalIgnore;
use crate::{Project, RelativePath};

pub struct SkipFile(Gitignore);

impl SkipFile {
    pub fn new(project: &Project) -> Result<Self> {
        let path = project.path().as_ref();
        let ignore_path = project.ignore_path();
        let mut gitignore_builder = GitignoreBuilder::new(path);
        let local = LocalIgnore::load(&ignore_path)?;
        let patterns = local.patterns;
        for glob in patterns {
            gitignore_builder.add_line(None, &glob)?;
        }
        Ok(Self(gitignore_builder.build()?))
    }

    pub fn is_skipped(&self, relative_path: &RelativePath) -> bool {
        if relative_path.as_str() == SKYSPELL_LOCAL_IGNORE {
            return true;
        }
        self.0
            .matched_path_or_any_parents(relative_path, false)
            .is_ignore()
    }
}

pub fn walk(project: &Project) -> Result<Walk> {
    Ok(WalkBuilder::new(project.path().as_ref()).build())
}

#[cfg(test)]
mod tests;
