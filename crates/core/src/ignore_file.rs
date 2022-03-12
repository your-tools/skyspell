use ignore::gitignore::Gitignore;
use ignore::Match;
use ignore::{Walk, WalkBuilder};

use crate::project::SKYSPELL_IGNORE_FILE;
use crate::{Project, RelativePath};

pub struct IgnoreFile(Gitignore);

impl IgnoreFile {
    pub fn new(project: &Project) -> Self {
        let ignore_path = project.ignore_path();
        let (ignore, _error) = Gitignore::new(ignore_path);
        // Note: _error will be Some(Err) if there's a invalid glob in
        // .skyspell-ignore for instance, but we don't care about that.
        Self(ignore)
    }

    pub fn is_ignored(&self, relative_path: &RelativePath) -> bool {
        if relative_path.as_str() == SKYSPELL_IGNORE_FILE {
            return true;
        }
        match self.0.matched(&relative_path, /*is-dir*/ false) {
            Match::Ignore(_) => true,
            Match::None | Match::Whitelist(_) => false,
        }
    }
}

pub fn walk(project: &Project) -> Walk {
    let ignore_path = project.ignore_path();
    WalkBuilder::new(project.path().as_ref())
        .add_custom_ignore_filename(ignore_path)
        .build()
}
