use std::path::Path;

use anyhow::Result;

use crate::{Project, RelativePath};

pub type ProjectId = i32;

pub struct ProjectInfo {
    id: ProjectId,
    path: String,
}

impl ProjectInfo {
    pub(crate) fn new(id: ProjectId, path: &str) -> Self {
        Self {
            id,
            path: path.to_string(),
        }
    }
    pub(crate) fn id(&self) -> ProjectId {
        self.id
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }
}
pub trait Repository {
    // Add the list of words to the global ignore list
    fn insert_ignored_words(&mut self, words: &[&str]) -> Result<()>;

    // Add word to the global ignore list
    fn ignore(&mut self, word: &str) -> Result<()>;
    // Is the word in the global ignore list?
    fn is_ignored(&self, word: &str) -> Result<bool>;

    // Add a new project
    fn new_project(&mut self, project: &Project) -> Result<ProjectId>;
    // Check if a project exists
    fn project_exists(&self, project: &Project) -> Result<bool>;
    // Create a project if it does not exist yet
    fn ensure_project(&mut self, project: &Project) -> Result<ProjectId> {
        if !self.project_exists(project)? {
            self.new_project(project)?;
        }
        self.get_project_id(project)
    }

    // Remove the given project from the list
    fn remove_project(&mut self, project_id: ProjectId) -> Result<()>;
    // Get project id
    fn get_project_id(&self, project: &Project) -> Result<ProjectId>;
    fn projects(&self) -> Result<Vec<ProjectInfo>>;

    fn clean(&mut self) -> Result<()> {
        for project in self.projects()? {
            let path = project.path();
            let path = Path::new(&path);
            let id = project.id();
            if !path.exists() {
                self.remove_project(id)?;
                println!("Removed non longer existing project: {}", path.display());
            }
        }
        Ok(())
    }

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
    fn ignore_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()>;
    // Is the word in the ignore list for the given project?
    fn is_ignored_for_project(&self, word: &str, project_id: ProjectId) -> Result<bool>;

    // Add word to the ignore list for the given project and path
    fn ignore_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()>;
    // Is the word in the ignore list for the given project and path?
    fn is_ignored_for_path(
        &self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool>;

    // Remove word from the global ignore list
    fn remove_ignored(&mut self, word: &str) -> Result<()>;
    // Remove word from the ignore list for the given extension
    fn remove_ignored_for_extension(&mut self, word: &str, extension: &str) -> Result<()>;
    // Remove word from the ignore list for the given path
    fn remove_ignored_for_path(
        &mut self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<()>;
    // Remove word from the ignore list for the given project
    fn remove_ignored_for_project(&mut self, word: &str, project_id: ProjectId) -> Result<()>;

    // Always skip the given file for the given project
    fn skip_path(&mut self, project_id: ProjectId, relative_path: &RelativePath) -> Result<()>;
    // Is the given path in the given project to be skipped ?
    fn is_skipped_path(&self, project: ProjectId, relative_path: &RelativePath) -> Result<bool>;
    // Remove file name from the skip list
    fn unskip_file_name(&mut self, file_name: &str) -> Result<()>;
    // Remove relative file path from the skip list
    fn unskip_path(&mut self, project_id: ProjectId, relative_path: &RelativePath) -> Result<()>;

    // Should this file be skipped ?
    fn should_skip(&self, project_id: ProjectId, relative_path: &RelativePath) -> Result<bool> {
        if let Some(f) = relative_path.file_name() {
            if self.is_skipped_file_name(&f)? {
                return Ok(true);
            }
        }

        if self.is_skipped_path(project_id, relative_path)? {
            return Ok(true);
        }

        Ok(false)
    }

    // Should this word be ignored?
    // This is called when a word is *not* found in the spelling dictionary.
    //
    // A word is ignored if:
    //   * it's in the global ignore list
    //   * the relative path has an extension and it's in the ignore list
    //     for this extension
    //   * it's in the ignore list for the project
    //   * it's in the ignore list for the relative path
    //
    // Otherwise, it's *not* ignored and the Checker will call handle_error()
    //
    fn should_ignore(
        &self,
        word: &str,
        project_id: ProjectId,
        relative_path: &RelativePath,
    ) -> Result<bool> {
        if self.is_ignored(word)? {
            return Ok(true);
        }

        if let Some(e) = relative_path.extension() {
            if self.is_ignored_for_extension(word, &e)? {
                return Ok(true);
            }
        }

        if self.is_ignored_for_project(word, project_id)? {
            return Ok(true);
        }

        self.is_ignored_for_path(word, project_id, relative_path)
    }
}

#[cfg(test)]
mod tests {
    use paste::paste;
    use tempdir::TempDir;

    use crate::sql::SQLRepository;
    use crate::tests::FakeRepository;

    use super::*;

    fn new_project(temp_dir: &TempDir, name: &'static str) -> Project {
        let temp_path = temp_dir.path();
        let project_path = temp_path.join(name);
        std::fs::create_dir(&project_path).unwrap();
        Project::open(&project_path).unwrap()
    }

    fn new_relative_path(project: &Project, name: &'static str) -> RelativePath {
        let rel_path = project.path().join(name);
        std::fs::write(&rel_path, "").unwrap();
        RelativePath::new(project, &rel_path).unwrap()
    }

    #[test]
    fn test_should_ignore_when_in_global_list() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");
        let relative_path = new_relative_path(&project, "foo");
        let mut repository = FakeRepository::new();

        repository.ignore("foo").unwrap();
        let project_id = repository.new_project(&project).unwrap();

        assert!(repository
            .should_ignore("foo", project_id, &relative_path)
            .unwrap());
    }

    #[test]
    fn test_should_ignore_when_in_list_for_extension() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");
        let foo_py = new_relative_path(&project, "foo.py");
        let foo_rs = new_relative_path(&project, "foo.rs");

        let mut repository = FakeRepository::new();
        let project_id = repository.new_project(&project).unwrap();
        repository.ignore_for_extension("foo", "py").unwrap();

        assert!(repository
            .should_ignore("foo", project_id, &foo_py)
            .unwrap());

        assert!(!repository
            .should_ignore("foo", project_id, &foo_rs)
            .unwrap());
    }

    #[test]
    fn test_should_ignore_when_in_project_list() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project_1 = new_project(&temp_dir, "project1");
        let foo_txt = new_relative_path(&project_1, "foo.txt");
        let project_2 = new_project(&temp_dir, "project2");
        let mut repository = FakeRepository::new();
        let project_id_1 = repository.new_project(&project_1).unwrap();
        let project_id_2 = repository.new_project(&project_2).unwrap();

        repository.ignore_for_project("foo", project_id_1).unwrap();

        assert!(repository
            .should_ignore("foo", project_id_1, &foo_txt)
            .unwrap());
        assert!(!repository
            .should_ignore("foo", project_id_2, &foo_txt)
            .unwrap());
    }

    #[test]
    fn test_should_skip_when_in_skipped_file_names_list() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");
        let cargo_lock = new_relative_path(&project, "Cargo.lock");
        let poetry_lock = new_relative_path(&project, "poetry.lock");

        let mut repository = FakeRepository::new();
        repository.new_project(&project).unwrap();
        repository.skip_file_name("Cargo.lock").unwrap();
        let project_id = repository.get_project_id(&project).unwrap();

        assert!(repository.should_skip(project_id, &cargo_lock).unwrap());
        assert!(!repository.should_skip(project_id, &poetry_lock).unwrap());
    }

    #[test]
    fn test_should_skip_when_in_skipped_paths() {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project_1 = new_project(&temp_dir, "project1");
        let foo_txt = new_relative_path(&project_1, "foo.txt");
        let other = new_relative_path(&project_1, "other");
        let project_2 = new_project(&temp_dir, "project2");

        let mut repository = FakeRepository::new();
        let project_id_1 = repository.new_project(&project_1).unwrap();
        let project_id_2 = repository.new_project(&project_2).unwrap();

        repository.skip_path(project_id_1, &foo_txt).unwrap();

        assert!(repository.should_skip(project_id_1, &foo_txt).unwrap());

        // Same project, other path
        assert!(!repository.should_skip(project_id_1, &other).unwrap());

        // Same file, other project
        assert!(!repository.should_skip(project_id_2, &foo_txt).unwrap());
    }

    // Given an identifier and a block, generate a test
    // for each implementation of the Repository trait
    // (SQLRepository and FakeRepository)
    macro_rules! make_tests {
        ($name:ident, ($repository:ident) => $test:block) => {
            paste! {
            fn $name(mut $repository: impl Repository) {
                $test
            }

            #[test]
            fn [<test_sql_repository $name>]() {
                let repository = SQLRepository::new(":memory:").unwrap();
                $name(repository)
            }

            #[test]
            fn [<test_fake_repository_ $name>]() {
                let repository = FakeRepository::new();
                $name(repository)
            }
            } // end paste
        };
    }

    make_tests!(insert_ignore, (repository) => {
        repository.ignore("foo").unwrap();

        assert!(repository.is_ignored("foo").unwrap());
        assert!(!repository.is_ignored("bar").unwrap());
    });

    make_tests!(lookup_extension, (repository) => {
        repository.ignore_for_extension("dict", "py").unwrap();

        assert!(repository.is_ignored_for_extension("dict", "py").unwrap());
    });

    make_tests!(insert_ignore_ignore_duplicates, (repository) => {
        repository.ignore("foo").unwrap();
        repository.ignore("foo").unwrap();
    });

    make_tests!(ignored_for_extension_duplicates, (repository) => {
        repository.ignore_for_extension("dict", "py").unwrap();
        repository.ignore_for_extension("dict", "py").unwrap();
    });

    make_tests!(create_project, (repository) => {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");

        assert!(!repository.project_exists(&project).unwrap());

        repository.new_project(&project).unwrap();
        assert!(repository.project_exists(&project).unwrap());
    });

    make_tests!(duplicate_projects, (repository) => {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");

        repository.new_project(&project).unwrap();
        assert!(repository.new_project(&project).is_err());
    });

    make_tests!(remove_project, (repository) => {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project1 = new_project(&temp_dir, "project1");
        let project2 = new_project(&temp_dir, "project2");
        let project3 = new_project(&temp_dir, "project3");
        repository.new_project(&project1).unwrap();
        let project2_id = repository.new_project(&project2).unwrap();
        repository.new_project(&project3).unwrap();

        repository.remove_project(project2_id).unwrap();

        assert!(!repository.project_exists(&project2).unwrap());
    });

    make_tests!(ignored_for_project, (repository) => {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");
        let other_project = new_project(&temp_dir, "other");

        repository.new_project(&project).unwrap();
        repository.new_project(&other_project).unwrap();

        let project_id = repository.get_project_id(&project).unwrap();
        let other_project_id = repository.get_project_id(&other_project).unwrap();
        repository.ignore_for_project("foo", project_id).unwrap();

        assert!(repository.is_ignored_for_project("foo", project_id).unwrap());
        assert!(!repository.is_ignored_for_project("foo", other_project_id).unwrap());
    });

    make_tests!(ignored_for_path, (repository) => {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");
        let foo_py = new_relative_path(&project, "foo.py");
        let foo_rs = new_relative_path(&project, "foo.rs");
        let other_project = new_project(&temp_dir, "other");
        repository.new_project(&project).unwrap();
        repository.new_project(&other_project).unwrap();

        let project_id = repository.get_project_id(&project).unwrap();
        let other_project_id = repository.get_project_id(&other_project).unwrap();
        repository.ignore_for_path("foo", project_id, &foo_py).unwrap();

        assert!(repository.is_ignored_for_path("foo", project_id, &foo_py).unwrap());
        // Same project, different file
        assert!(!repository.is_ignored_for_path("foo", project_id, &foo_rs).unwrap());
        // Same file, different project
        assert!(!repository.is_ignored_for_path("foo", other_project_id, &foo_py).unwrap());
    });

    make_tests!(skip_file_name, (repository) => {
        assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());

        repository.skip_file_name("Cargo.lock").unwrap();
        assert!(repository.is_skipped_file_name("Cargo.lock").unwrap());
    });

    make_tests!(skip_path, (repository) => {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");
        let test_txt = new_relative_path(&project, "test.txt");

        let project_id = repository.new_project(&project).unwrap();
        assert!(!repository.is_skipped_path(project_id, &test_txt).unwrap());

        repository.skip_path(project_id, &test_txt).unwrap();

        assert!(repository.is_skipped_path(project_id, &test_txt).unwrap());
    });

    make_tests!(remove_ignored, (repository) => {
        repository.ignore("foo").unwrap();

        repository.remove_ignored("foo").unwrap();

        assert!(!repository.is_ignored("foo").unwrap());
    });

    make_tests!(remove_ignored_for_extension, (repository) => {
        repository.ignore_for_extension("foo", "py").unwrap();

        repository.remove_ignored_for_extension("foo", "py").unwrap();

        assert!(!repository.is_ignored_for_extension("foo", "py").unwrap());

    });

    make_tests!(remove_ignored_for_path, (repository) => {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");
        let project_id = repository.new_project(&project).unwrap();
        let foo_py = new_relative_path(&project, "foo.py");
        repository.ignore_for_path("foo", project_id, &foo_py).unwrap();

        repository.remove_ignored_for_path("foo", project_id, &foo_py).unwrap();

        assert!(!repository.is_ignored_for_path("foo", project_id, &foo_py).unwrap());
    });

    make_tests!(remove_ignored_for_project, (repository) => {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");
        repository.new_project(&project).unwrap();
        let project_id = repository.get_project_id(&project).unwrap();
        repository.ignore_for_project("foo", project_id).unwrap();

        repository.remove_ignored_for_project("foo", project_id).unwrap();

        assert!(!repository.is_ignored_for_project("foo", project_id).unwrap());
    });

    make_tests!(unskip_file_name, (repository) => {
        repository.skip_file_name("Cargo.lock").unwrap();

        repository.unskip_file_name("Cargo.lock").unwrap();

        assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());
    });

    make_tests!(unskip_path, (repository) => {
        let temp_dir = tempdir::TempDir::new("test-skyspell").unwrap();
        let project = new_project(&temp_dir, "project");
        let project_id = repository.new_project(&project).unwrap();
        let foo_py = new_relative_path(&project, "foo.py");
        repository.skip_path(project_id, &foo_py).unwrap();

        repository.unskip_path(project_id, &foo_py).unwrap();

        assert!(!repository.is_skipped_path(project_id, &foo_py).unwrap());
    });
}
