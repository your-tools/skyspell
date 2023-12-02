use anyhow::Result;

use ignore::gitignore::{Gitignore, GitignoreBuilder};
use ignore::Match;
use ignore::{Walk, WalkBuilder};

use crate::project::SKYSPELL_CONFIG_FILE;
use crate::IgnoreConfig;
use crate::{Project, RelativePath};

pub struct SkipFile(Gitignore);

impl SkipFile {
    pub fn new(project: &Project) -> Result<Self> {
        let path = project.path().as_ref();
        let ignore_path = project.ignore_path();
        let mut gitignore_builder = GitignoreBuilder::new(path);
        if ignore_path.exists() {
            let ignore_config = IgnoreConfig::open(&ignore_path)?;
            for glob in ignore_config.patterns() {
                gitignore_builder.add_line(None, glob)?;
            }
        }
        Ok(Self(gitignore_builder.build()?))
    }

    pub fn is_skipped(&self, relative_path: &RelativePath) -> bool {
        if relative_path.as_str() == SKYSPELL_CONFIG_FILE {
            return true;
        }
        match self.0.matched(relative_path, /*is-dir*/ false) {
            Match::Ignore(_) => true,
            Match::None | Match::Whitelist(_) => false,
        }
    }
}

pub fn walk(project: &Project) -> Result<Walk> {
    Ok(WalkBuilder::new(project.path().as_ref()).build())
}
