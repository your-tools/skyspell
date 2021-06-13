use anyhow::{anyhow, Result};
use std::path::Path;

pub trait Repository {
    // Add the list of words to the global ignore list
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()>;

    // Add word to the global ignore list
    fn ignore(&mut self, word: &str) -> Result<()>;
    // Is the word in the global ignore list?
    fn is_ignored(&self, word: &str) -> Result<bool>;

    fn new_project(&mut self, path: &Path) -> Result<()>;
    fn project_exists(&self, path: &Path) -> Result<bool>;

    // Always skip this file name - to be used with Cargo.lock, yarn.lock
    // and the like
    fn skip_file_name(&mut self, file_name: &str) -> Result<()>;
    // Is this file name to be skipped ?
    fn is_skipped_file_name(&self, file_name: &str) -> Result<bool>;

    // Add word to the ignore list for the given extension
    fn ignore_for_extension(&mut self, word: &str, extension: &str) -> Result<()>;
    // Is the word in the ignore list for the given extension?
    fn is_ignored_for_extension(&self, word: &str, extension: &str) -> Result<bool>;

    // Add word to the ignore list for the given project
    fn ignore_for_project(&mut self, word: &str, project_path: &Path) -> Result<()>;
    // Is the word in the ignore list for the given project?
    fn is_ignored_for_project(&self, word: &str, project_path: &Path) -> Result<bool>;

    // Add word to the ignore list for the given project and path
    fn ignore_for_path(
        &mut self,
        word: &str,
        project_path: &Path,
        relative_path: &Path,
    ) -> Result<()>;
    // Add word to the ignore list for the given project and path
    fn is_ignored_for_path(
        &self,
        word: &str,
        project_path: &Path,
        relative_path: &Path,
    ) -> Result<bool>;

    // Always skip the given file for the given project
    fn skip_path(&mut self, project_path: &Path, relative_path: &Path) -> Result<()>;
    // Is the given path in the given project to be skipped ?
    fn is_skipped_path(&self, project_path: &Path, relative_path: &Path) -> Result<bool>;

    // Should this file be skipped ?
    fn should_skip(&self, project_path: Option<&Path>, relative_path: &Path) -> Result<bool> {
        let file_name = relative_path.file_name().map(|x| x.to_string_lossy());
        if let Some(f) = file_name {
            if self.is_skipped_file_name(&f)? {
                return Ok(true);
            }
        }

        if let (Some(p), q) = (project_path, relative_path) {
            return self.is_skipped_path(p, q);
        }

        Ok(false)
    }

    // Should this word be ignored?
    fn should_ignore(&self, error: &str, project_path: Option<&Path>, path: &Path) -> Result<bool> {
        let extension = path.extension().and_then(|x| x.to_str());

        if let Some(e) = extension {
            if self.is_ignored_for_extension(error, e)? {
                return Ok(true);
            }
        }

        if let Some(project_path) = project_path {
            if self.is_ignored_for_project(error, project_path)? {
                return Ok(true);
            }

            let relative_path = pathdiff::diff_paths(path, &project_path).ok_or_else(|| {
                anyhow!(
                    "Could not build relative path from {} to {}",
                    path.display(),
                    project_path.display()
                )
            })?;
            if self.is_ignored_for_path(error, project_path, &relative_path)? {
                return Ok(true);
            }
        }

        self.is_ignored(error)
    }
}

#[cfg(test)]
mod tests {
    use paste::paste;

    use crate::tests::FakeRepository;
    use crate::Db;

    use super::*;

    #[test]
    fn test_should_ignore_when_in_global_list() {
        let mut repo = FakeRepository::new();

        repo.ignore("foo").unwrap();

        assert!(repo
            .should_ignore("foo", None, &Path::new("foo.txt"))
            .unwrap());
    }

    #[test]
    fn test_should_ignore_when_in_list_for_extension() {
        let mut repo = FakeRepository::new();

        repo.ignore_for_extension("foo", "py").unwrap();

        assert!(repo
            .should_ignore("foo", None, &Path::new("foo.py"))
            .unwrap());

        assert!(!repo
            .should_ignore("foo", None, &Path::new("foo.txt"))
            .unwrap());
    }

    #[test]
    fn test_should_ignore_when_in_project_list() {
        let mut repo = FakeRepository::new();
        repo.new_project(Path::new("/path/to/project")).unwrap();

        repo.ignore_for_project("foo", Path::new("/path/to/project"))
            .unwrap();

        assert!(repo
            .should_ignore(
                "foo",
                Some(Path::new("/path/to/project")),
                &Path::new("/path/to/project/foo.py")
            )
            .unwrap());

        assert!(!repo
            .should_ignore(
                "foo",
                Some(Path::new("/path/to/other/project")),
                &Path::new("/path/to/project/foo.txt")
            )
            .unwrap());
    }

    #[test]
    fn test_should_skip_when_in_skip_list() {
        let mut repo = FakeRepository::new();
        repo.skip_file_name("Cargo.lock").unwrap();

        assert!(repo
            .should_skip(None, &Path::new("/path/to/Cargo.lock"))
            .unwrap());

        assert!(!repo
            .should_skip(None, &Path::new("/path/to/poetry.lock"))
            .unwrap());
    }

    #[test]
    fn test_should_skip_when_in_skipped_paths() {
        let mut repo = FakeRepository::new();
        repo.new_project(&Path::new("/path/to/project")).unwrap();
        repo.skip_path(&Path::new("/path/to/project"), &Path::new("test.txt"))
            .unwrap();

        assert!(repo
            .should_skip(Some(&Path::new("/path/to/project")), &Path::new("test.txt"))
            .unwrap());

        // Same project, other path
        assert!(!repo
            .should_skip(Some(&Path::new("/path/to/project")), &Path::new("test.py"))
            .unwrap());

        // Same file, other project
        assert!(!repo
            .should_skip(
                Some(&Path::new("/path/to/other/project")),
                &Path::new("test.txt")
            )
            .unwrap());

        // Same file, no project
        assert!(!repo.should_skip(None, &Path::new("test.txt")).unwrap());
    }

    // Given an identifier and a block, generate a test
    // for each implementation of the Repo trait
    // (Db and FakeRepo)
    macro_rules! make_tests {
        ($name:ident, ($repo:ident) => $test:block) => {
            paste! {
            fn $name(mut $repo: impl Repository) {
                $test
            }

            #[test]
            fn [<test_db_ $name>]() {
                let db = Db::connect(":memory:").unwrap();
                $name(db)
            }

            #[test]
            fn [<test_fake_repo_ $name>]() {
                let repo = FakeRepository::new();
                $name(repo)
            }
            } // end paste
        };
    }

    make_tests!(insert_ignore, (repo) => {
        repo.ignore("foo").unwrap();

        assert!(repo.is_ignored("foo").unwrap());
        assert!(!repo.is_ignored("bar").unwrap());
    });

    make_tests!(lookup_extension, (repo) => {
        repo.ignore_for_extension("dict", "py").unwrap();

        assert!(repo.is_ignored_for_extension("dict", "py").unwrap());
    });

    make_tests!(insert_ignore_ignore_duplicates, (repo) => {
        repo.ignore("foo").unwrap();
        repo.ignore("foo").unwrap();
    });

    make_tests!(ignored_for_extension_duplicates, (repo) => {
        repo.ignore_for_extension("dict", "py").unwrap();
        repo.ignore_for_extension("dict", "py").unwrap();
    });

    make_tests!(create_project, (repo)=>{
        assert!(!repo.project_exists(&Path::new("/path/to/project")).unwrap());

        repo.new_project(&Path::new("/path/to/project")).unwrap();
        assert!(repo.project_exists(&Path::new("/path/to/project")).unwrap());
    });

    make_tests!(duplicate_projects, (repo) => {

        repo.new_project(&Path::new("/path/to/project")).unwrap();
        assert!(repo.new_project(&Path::new("/path/to/project")).is_err());
    });

    make_tests!(ignored_for_project, (repo) => {
        repo.new_project(&Path::new("/path/to/project")).unwrap();
        repo.new_project(&Path::new("/path/to/other/project")).unwrap();

        repo.ignore_for_project("foo", &Path::new("/path/to/project")).unwrap();

        assert!(repo.is_ignored_for_project("foo", &Path::new("/path/to/project")).unwrap());
        assert!(!repo.is_ignored_for_project("foo", &Path::new("/path/to/other/project")).unwrap());
    });

    make_tests!(ignored_for_path, (repo) => {
        repo.new_project(&Path::new("/path/to/project")).unwrap();
        repo.new_project(&Path::new("/path/to/other/project")).unwrap();
        repo.ignore_for_path("foo", &Path::new("/path/to/project"), &Path::new("foo.py")).unwrap();

        assert!(repo.is_ignored_for_path("foo", &Path::new("/path/to/project"), &Path::new("foo.py")).unwrap());
        assert!(!repo.is_ignored_for_path("foo", &Path::new("/path/to/other/project"), &Path::new("bar.py")).unwrap());
    });

    make_tests!(skip_file_name, (repo) => {
        assert!(!repo.is_skipped_file_name("Cargo.lock").unwrap());

        repo.skip_file_name("Cargo.lock").unwrap();
        assert!(repo.is_skipped_file_name("Cargo.lock").unwrap());
    });

    make_tests!(skip_path, (repo) => {
        repo.new_project(&Path::new("/path/to/project")).unwrap();
        assert!(!repo.is_skipped_path(&Path::new("/path/to/project"), &Path::new("test.txt")).unwrap());

        repo.skip_path(&Path::new("/path/to/project"), &Path::new("test.txt")).unwrap();

        assert!(repo.is_skipped_path(&Path::new("/path/to/project"), &Path::new("test.txt")).unwrap());
    });
}
