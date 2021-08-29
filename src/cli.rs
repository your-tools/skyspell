use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Clap;

use crate::kak;
use crate::kak::io::KakouneIO;
use crate::StandardIO;
use crate::TokenProcessor;
use crate::{Checker, InteractiveChecker, NonInteractiveChecker};
use crate::{ConsoleInteractor, Dictionary, Repository};
use crate::{ProjectPath, RelativePath};

pub fn run<D: Dictionary, R: Repository>(opts: Opts, dictionary: D, repository: R) -> Result<()> {
    match opts.action {
        Action::Add(opts) => add(repository, opts),
        Action::Remove(opts) => remove(repository, opts),
        Action::Check(opts) => check(repository, dictionary, opts),
        Action::Clean => clean(repository),
        Action::ImportPersonalDict(opts) => import_personal_dict(repository, opts),
        Action::Suggest(opts) => suggest(dictionary, opts),
        Action::Skip(opts) => skip(repository, opts),
        Action::Unskip(opts) => unskip(repository, opts),
        Action::Kak(opts) => {
            let io = StandardIO;
            let kakoune_io = KakouneIO::new(io);
            if let Err(e) = kak::cli::run(repository, dictionary, kakoune_io, opts) {
                println!("echo -markup {{Error}}{}", e);
                return Err(e);
            }
            Ok(())
        }
    }
}

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
pub struct Opts {
    #[clap(
        long,
        about = "Language to use",
        long_about = "Language to use - must match an installed dictionary for one of Enchant's providers"
    )]
    pub lang: Option<String>,

    #[clap(long, about = "Path of the ignore repository")]
    pub db_path: Option<String>,

    #[clap(subcommand)]
    action: Action,
}

#[derive(Clap)]
enum Action {
    #[clap(about = "Add word to one of the ignore lists")]
    Add(AddOpts),
    #[clap(about = "Remove word from one of the ignore lists")]
    Remove(RemoveOpts),
    #[clap(about = "Check files for spelling errors")]
    Check(CheckOpts),
    #[clap(about = "Clean repository")]
    Clean,
    #[clap(about = "Import a personal dictionary")]
    ImportPersonalDict(ImportPersonalDictOpts),
    #[clap(about = "Suggest replacements for the given error")]
    Suggest(SuggestOpts),
    #[clap(about = "Add path tho the given skipped list")]
    Skip(SkipOpts),
    #[clap(about = "Remove path from the given skipped list")]
    Unskip(UnskipOpts),

    #[clap(about = "Kakoune actions")]
    Kak(kak::cli::Opts),
}

#[derive(Clap)]
struct AddOpts {
    #[clap(long, about = "Project path")]
    project_path: Option<PathBuf>,

    word: String,

    #[clap(long, about = "Add word to the ignore list for the given extension")]
    extension: Option<String>,

    #[clap(long, about = "Add word to the ignore list for the given path")]
    relative_path: Option<PathBuf>,
}

#[derive(Clap)]
struct CheckOpts {
    #[clap(long, about = "Project path")]
    project_path: PathBuf,

    #[clap(long)]
    non_interactive: bool,

    #[clap(about = "List of paths to check")]
    sources: Vec<PathBuf>,
}

#[derive(Clap)]
struct ImportPersonalDictOpts {
    #[clap(long)]
    personal_dict_path: PathBuf,
}

#[derive(Clap)]
struct SkipOpts {
    #[clap(long, about = "Project path")]
    project_path: Option<PathBuf>,

    #[clap(long, about = "File path to skip")]
    relative_path: Option<PathBuf>,

    #[clap(long, about = "File name to skip")]
    file_name: Option<String>,
}

#[derive(Clap)]
struct UnskipOpts {
    #[clap(long, about = "Project path")]
    project_path: Option<PathBuf>,

    #[clap(long, about = "File path to unskip")]
    relative_path: Option<PathBuf>,

    #[clap(long, about = "File name to unskip")]
    file_name: Option<String>,
}

#[derive(Clap)]
struct SuggestOpts {
    word: String,
}

#[derive(Clap)]
struct RemoveOpts {
    #[clap(long, about = "Project path")]
    project_path: Option<PathBuf>,

    #[clap(
        long,
        about = "Remove word from the ignore list for the given extension"
    )]
    extension: Option<String>,
    #[clap(long, about = "Remove word from the ignore list for the given path")]
    relative_path: Option<PathBuf>,

    word: String,
}

fn add(mut repository: impl Repository, opts: AddOpts) -> Result<()> {
    let word = &opts.word;
    match (opts.project_path, opts.relative_path, opts.extension) {
        (None, None, None) => repository.ignore(word),
        (None, _, Some(e)) => repository.ignore_for_extension(word, &e),
        (Some(project_path), Some(relative_path), None) => {
            let project = ProjectPath::open(&project_path)?;
            let project_id = repository.ensure_project(&project)?;
            let relative_path = RelativePath::new(&project, &relative_path)?;
            repository.ignore_for_path(word, project_id, &relative_path)
        }
        (Some(project_path), None, None) => {
            let project = ProjectPath::open(&project_path)?;
            repository.ensure_project(&project)?;
            let project_id = repository.get_project_id(&project)?;
            repository.ignore_for_project(word, project_id)
        }
        (None, Some(_), None) => bail!("Cannot use --relative-path without --project-path"),
        (Some(_), _, Some(_)) => bail!("--extension is incompatible with --project-path"),
    }
}

fn remove(mut repository: impl Repository, opts: RemoveOpts) -> Result<()> {
    let word = &opts.word;
    match (opts.project_path, opts.relative_path, opts.extension) {
        (None, None, None) => repository.remove_ignored(word),
        (None, _, Some(e)) => repository.remove_ignored_for_extension(word, &e),
        (Some(project_path), Some(relative_path), None) => {
            let project = ProjectPath::open(&project_path)?;
            let project_id = repository.get_project_id(&project)?;
            let relative_path = RelativePath::new(&project, &relative_path)?;
            repository.remove_ignored_for_path(word, project_id, &relative_path)
        }
        (Some(project_path), None, None) => {
            let project = ProjectPath::open(&project_path)?;
            let project_id = repository.get_project_id(&project)?;
            repository.remove_ignored_for_project(word, project_id)
        }
        (None, Some(_), None) => bail!("Cannot use --relative-path without --project-path"),
        (Some(_), _, Some(_)) => bail!("--extension is incompatible with --project-path"),
    }
}

fn check(repository: impl Repository, dictionary: impl Dictionary, opts: CheckOpts) -> Result<()> {
    let project = ProjectPath::open(&opts.project_path)?;
    println!("Checking project {} for spelling errors", project);

    let interactive = !opts.non_interactive;

    match interactive {
        false => {
            let mut checker = NonInteractiveChecker::new(project, dictionary, repository)?;
            check_with(&mut checker, opts)
        }
        true => {
            let interactor = ConsoleInteractor;
            let mut checker = InteractiveChecker::new(project, interactor, dictionary, repository)?;
            check_with(&mut checker, opts)
        }
    }
}

fn check_with<C>(checker: &mut C, opts: CheckOpts) -> Result<()>
where
    C: Checker<Context = (usize, usize)>,
{
    if opts.sources.is_empty() {
        println!("No path given - nothing to do");
    }

    let mut skipped = 0;
    let mut checked = 0;
    for path in &opts.sources {
        let relative_path = checker.to_relative_path(path)?;
        if checker.should_skip(&relative_path)? {
            skipped += 1;
            continue;
        }

        let token_processor = TokenProcessor::new(path)?;
        token_processor.each_token(|word, line, column| {
            checker.handle_token(word, &relative_path, &(line, column))
        })?;
        checked += 1;
    }

    match skipped {
        1 => println!("Skipped one file"),
        x if x >= 2 => println!("Skipped {} files", x),
        _ => (),
    }

    checker.success()?;

    println!("Success. {} files checked.", checked);

    Ok(())
}

fn clean(mut repository: impl Repository) -> Result<()> {
    repository.clean()
}

fn import_personal_dict(
    mut repository: impl Repository,
    opts: ImportPersonalDictOpts,
) -> Result<()> {
    let dict = std::fs::read_to_string(&opts.personal_dict_path)?;
    let words: Vec<&str> = dict.split_ascii_whitespace().collect();
    repository.insert_ignored_words(&words)?;

    Ok(())
}

fn skip(mut repository: impl Repository, opts: SkipOpts) -> Result<()> {
    match (opts.project_path, opts.relative_path, opts.file_name) {
        (Some(project_path), Some(relative_path), None) => {
            let project = ProjectPath::open(&project_path)?;
            let project_id = repository.ensure_project(&project)?;
            let relative_path = RelativePath::new(&project, &relative_path)?;
            repository.skip_path(project_id, &relative_path)
        }
        (_, None, Some(file_name)) => repository.skip_file_name(&file_name),
        (_, _, _) => {
            bail!("Either use --file-name OR --project-path and --relative-path")
        }
    }
}

fn unskip(mut repository: impl Repository, opts: UnskipOpts) -> Result<()> {
    match (opts.project_path, opts.relative_path, opts.file_name) {
        (Some(project_path), Some(relative_path), None) => {
            let project = ProjectPath::open(&project_path)?;
            let project_id = repository.get_project_id(&project)?;
            let relative_path = RelativePath::new(&project, &relative_path)?;
            repository.unskip_path(project_id, &relative_path)
        }
        (_, None, Some(file_name)) => repository.unskip_file_name(&file_name),
        (_, _, _) => {
            bail!("Either use --file-name OR --project-path and --relative-path")
        }
    }
}

fn suggest(dictionary: impl Dictionary, opts: SuggestOpts) -> Result<()> {
    let word = &opts.word;
    if dictionary.check(word)? {
        return Ok(());
    }

    let suggestions = dictionary.suggest(word);

    for suggestion in suggestions.iter() {
        println!("{}", suggestion);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::sql::SQLRepository;
    use crate::tests::FakeDictionary;
    use crate::{ProjectPath, RelativePath};

    use tempdir::TempDir;

    fn open_repository(temp_dir: &TempDir) -> SQLRepository {
        SQLRepository::new(&TestApp::db_path(temp_dir)).unwrap()
    }

    fn open_project(temp_dir: &TempDir, name: &str) -> ProjectPath {
        let path = temp_dir.path().join(name);
        std::fs::create_dir_all(&path).unwrap();
        ProjectPath::open(&path).unwrap()
    }

    struct TestApp {
        dictionary: FakeDictionary,
        repository: SQLRepository,
    }

    impl TestApp {
        fn new(temp_dir: &TempDir) -> Self {
            let dictionary = FakeDictionary::new();
            let db_path = Self::db_path(temp_dir);
            let repository = SQLRepository::new(&db_path).unwrap();
            Self {
                dictionary,
                repository,
            }
        }

        fn open_project(&mut self, temp_dir: &TempDir, project_name: &str) -> ProjectPath {
            open_project(temp_dir, project_name)
        }

        fn ensure_file(
            temp_dir: &TempDir,
            project_name: &str,
            file_name: &str,
        ) -> (PathBuf, RelativePath) {
            let project = open_project(temp_dir, project_name);
            let full_path = temp_dir.path().join(file_name);
            std::fs::write(&full_path, "").unwrap();
            (
                full_path.clone(),
                RelativePath::new(&project, &full_path).unwrap(),
            )
        }

        fn db_path(temp_dir: &TempDir) -> String {
            temp_dir
                .path()
                .join("tests.db")
                .to_string_lossy()
                .to_string()
        }

        fn run(self, args: &[&str]) -> Result<()> {
            let mut with_arg0 = vec!["skyspell"];
            with_arg0.extend(args);
            let opts = Opts::parse_from(with_arg0);
            super::run(opts, self.dictionary, self.repository)
        }
    }

    #[test]
    fn test_add_global() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let app = TestApp::new(&temp_dir);

        app.run(&["add", "foo"]).unwrap();

        let repository = open_repository(&temp_dir);
        assert!(repository.is_ignored("foo").unwrap());
    }

    #[test]
    fn test_add_for_project_happy() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        let project = app.open_project(&temp_dir, "project");

        app.run(&["add", "foo", "--project-path", &project.as_str()])
            .unwrap();

        let repository = open_repository(&temp_dir);
        let project_id = repository.get_project_id(&project).unwrap();
        assert!(repository
            .is_ignored_for_project("foo", project_id)
            .unwrap());
    }

    #[test]
    fn test_add_for_extension() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let app = TestApp::new(&temp_dir);
        TestApp::ensure_file(&temp_dir, "project", "foo.py");

        app.run(&["add", "foo", "--extension", "py"]).unwrap();

        let repository = open_repository(&temp_dir);
        assert!(repository.is_ignored_for_extension("foo", "py").unwrap());
    }

    #[test]
    fn test_add_for_relative_path() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        let (full_path, rel_path) = TestApp::ensure_file(&temp_dir, "project", "foo.txt");
        let project = app.open_project(&temp_dir, "project");

        app.run(&[
            "add",
            "foo",
            "--project-path",
            &project.as_str(),
            "--relative-path",
            &full_path.to_string_lossy(),
        ])
        .unwrap();

        let repository = open_repository(&temp_dir);
        let project_id = repository.get_project_id(&project).unwrap();
        assert!(repository
            .is_ignored_for_path("foo", project_id, &rel_path)
            .unwrap());
    }

    #[test]
    fn test_remove_global() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.repository.ignore("foo").unwrap();

        app.run(&["remove", "foo"]).unwrap();

        let repository = open_repository(&temp_dir);
        assert!(!repository.is_ignored("foo").unwrap());
    }

    #[test]
    fn test_remove_for_project() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        let project = app.open_project(&temp_dir, "project");
        app.repository.new_project(&project).unwrap();
        let project_id = app.repository.get_project_id(&project).unwrap();
        app.repository
            .ignore_for_project("foo", project_id)
            .unwrap();

        app.run(&["remove", "foo", "--project-path", &project.as_str()])
            .unwrap();

        let repository = open_repository(&temp_dir);
        let project_id = repository.get_project_id(&project).unwrap();
        assert!(!repository
            .is_ignored_for_project("foo", project_id)
            .unwrap());
    }

    #[test]
    fn test_remove_for_relative_path() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        let (full_path, rel_path) = TestApp::ensure_file(&temp_dir, "project", "foo.txt");
        let project = app.open_project(&temp_dir, "project");
        let project_id = app.repository.new_project(&project).unwrap();
        app.repository
            .ignore_for_path("foo", project_id, &rel_path)
            .unwrap();

        app.run(&[
            "remove",
            "foo",
            "--project-path",
            &project.as_str(),
            "--relative-path",
            &full_path.to_string_lossy(),
        ])
        .unwrap();

        let repository = open_repository(&temp_dir);
        let project_id = repository.get_project_id(&project).unwrap();
        assert!(!repository
            .is_ignored_for_path("foo", project_id, &rel_path)
            .unwrap());
    }

    #[test]
    fn test_remove_for_extension() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        TestApp::ensure_file(&temp_dir, "project", "foo.py");
        app.repository.ignore_for_extension("foo", "py").unwrap();

        app.run(&["remove", "foo", "--extension", "py"]).unwrap();

        let repository = open_repository(&temp_dir);
        assert!(!repository.is_ignored_for_extension("foo", "py").unwrap());
    }

    #[test]
    fn test_check_errors_in_two_files() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        let project = app.open_project(&temp_dir, "project");
        let (foo_full, _) = TestApp::ensure_file(&temp_dir, "project", "foo.md");
        let (bar_full, _) = TestApp::ensure_file(&temp_dir, "project", "bar.md");
        std::fs::write(&foo_full, "This is foo").unwrap();
        std::fs::write(&bar_full, "This is bar and it contains baz").unwrap();
        for word in &["This", "is", "and", "it", "contains"] {
            app.dictionary.add_known(word);
        }

        let err = app
            .run(&[
                "check",
                "--non-interactive",
                "--project-path",
                &project.as_str(),
                &bar_full.to_string_lossy(),
                &foo_full.to_string_lossy(),
            ])
            .unwrap_err();

        assert!(err.to_string().contains("spelling errors"))
    }

    #[test]
    fn test_check_happy() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        let project = app.open_project(&temp_dir, "project");
        let (foo_full, _) = TestApp::ensure_file(&temp_dir, "project", "foo.md");
        let (bar_full, _) = TestApp::ensure_file(&temp_dir, "project", "bar.md");
        std::fs::write(&foo_full, "This is fine").unwrap();
        std::fs::write(&bar_full, "This is also fine").unwrap();
        for word in &["This", "is", "also", "fine"] {
            app.dictionary.add_known(word);
        }

        app.run(&[
            "check",
            "--non-interactive",
            "--project-path",
            &project.as_str(),
            &bar_full.to_string_lossy(),
            &foo_full.to_string_lossy(),
        ])
        .unwrap();
    }

    #[test]
    fn test_skip_relative_path() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        let (full_path, rel_path) = TestApp::ensure_file(&temp_dir, "project", "foo.txt");
        let project = app.open_project(&temp_dir, "project");

        app.run(&[
            "skip",
            "--project-path",
            &project.as_str(),
            "--relative-path",
            &full_path.to_string_lossy(),
        ])
        .unwrap();

        let repository = open_repository(&temp_dir);
        let project_id = repository.get_project_id(&project).unwrap();
        assert!(repository.is_skipped_path(project_id, &rel_path).unwrap());
    }

    #[test]
    fn test_skip_file_name() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let app = TestApp::new(&temp_dir);

        app.run(&["skip", "--file-name", "Cargo.lock"]).unwrap();

        let repository = open_repository(&temp_dir);
        assert!(repository.is_skipped_file_name("Cargo.lock").unwrap());
    }

    #[test]
    fn test_unskip_relative_path() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        let (full_path, rel_path) = TestApp::ensure_file(&temp_dir, "project", "foo.txt");
        let project = app.open_project(&temp_dir, "project");
        let project_id = app.repository.new_project(&project).unwrap();
        app.repository.skip_path(project_id, &rel_path).unwrap();

        app.run(&[
            "unskip",
            "--project-path",
            &project.as_str(),
            "--relative-path",
            &full_path.to_string_lossy(),
        ])
        .unwrap();

        let repository = open_repository(&temp_dir);
        assert!(!repository.is_skipped_path(project_id, &rel_path).unwrap());
    }

    #[test]
    fn test_unskip_file_name() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.repository.skip_file_name("Cargo.lock").unwrap();

        app.run(&["unskip", "--file-name", "Cargo.lock"]).unwrap();

        let repository = open_repository(&temp_dir);
        assert!(!repository.is_skipped_file_name("Cargo.lock").unwrap());
    }

    #[test]
    fn test_suggest() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        app.dictionary
            .add_suggestions("hel", &["hello".to_string(), "hell".to_string()]);

        app.run(&["suggest", "hel"]).unwrap();
    }

    #[test]
    fn test_clean() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut app = TestApp::new(&temp_dir);
        let project1 = app.open_project(&temp_dir, "project1");
        app.repository.new_project(&project1).unwrap();
        let project2 = app.open_project(&temp_dir, "project2");
        app.repository.new_project(&project2).unwrap();
        let before = app.repository.projects().unwrap();

        std::fs::remove_dir_all(&project2.as_ref()).unwrap();

        app.run(&["clean"]).unwrap();

        let repository = open_repository(&temp_dir);
        let after = repository.projects().unwrap();

        assert_eq!(
            before.len() - after.len(),
            1,
            "Should have removed one project"
        );
    }
}
