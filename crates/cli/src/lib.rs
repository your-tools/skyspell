use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::*;

use skyspell_core::ignore_file::walk;
use skyspell_core::Checker;
use skyspell_core::Dictionary;
use skyspell_core::EnchantDictionary;
use skyspell_core::IgnoreConfig;
use skyspell_core::StorageBackend;
use skyspell_core::TokenProcessor;
use skyspell_core::{get_default_db_path, SQLRepository};
use skyspell_core::{Project, ProjectPath, SKYSPELL_IGNORE_FILE};

mod checkers;
pub mod interactor;
pub use checkers::{InteractiveChecker, NonInteractiveChecker};
pub use interactor::{ConsoleInteractor, Interactor};

#[macro_export]
macro_rules! info_1 {
    ($($arg:tt)*) => ({
        println!("{} {}", "::".bold().blue(), format!($($arg)*));
    })
}

#[macro_export]
macro_rules! info_2 {
    ($($arg:tt)*) => ({
        println!("{} {}", "=>".bold().blue(), format!($($arg)*));
    })
}

#[macro_export]
macro_rules! info_3 {
    ($($arg:tt)*) => ({
        println!("{} {}", "*".bold().blue(), format!($($arg)*));
    })
}

#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => ({
    eprintln!("{} {}", "Error:".red(), format!($($arg)*));
    })
}

#[derive(Parser)]
#[clap(version)]
pub struct Opts {
    #[clap(long, help = "Language to use")]
    pub lang: Option<String>,

    #[clap(long, help = "Path of the ignore repository")]
    pub db_path: Option<String>,

    #[clap(long, help = "Project path")]
    project_path: Option<PathBuf>,

    #[clap(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    #[clap(about = "Add word to one of the ignore lists")]
    Add(AddOpts),
    #[clap(about = "Remove word from one of the ignore lists")]
    Remove(RemoveOpts),
    #[clap(about = "Check files for spelling errors")]
    Check(CheckOpts),
    #[clap(about = "Clean repository")]
    Clean,
    #[clap(about = "Suggest replacements for the given error")]
    Suggest(SuggestOpts),
    #[clap(about = "Undo last operation")]
    Undo,
}

#[derive(Parser)]
struct AddOpts {
    #[clap(help = "The word to add")]
    word: String,

    #[clap(long, help = "Add word to the ignore list for the current project")]
    project: bool,

    #[clap(long, help = "Add word to the ignore list for the given extension")]
    extension: Option<String>,

    #[clap(long, help = "Add word to the ignore list for the given path")]
    relative_path: Option<PathBuf>,
}

#[derive(Parser)]
struct CheckOpts {
    #[clap(
        long,
        help = "Don't ask what to do for each unknown word, instead just print the whole list - useful for continuous integration and other scripts"
    )]
    non_interactive: bool,
}

#[derive(Parser)]
struct SuggestOpts {
    word: String,
}

#[derive(Parser)]
struct RemoveOpts {
    #[clap(help = "The word to remove")]
    word: String,

    #[clap(
        long,
        help = "Remove word from the ignore list for the current project"
    )]
    project: bool,

    #[clap(
        long,
        help = "Remove word from the ignore list for the given extension"
    )]
    extension: Option<String>,

    #[clap(long, help = "Remove word from the ignore list for the given path")]
    relative_path: Option<PathBuf>,
}

fn add(project: Project, mut storage_backend: StorageBackend, opts: &AddOpts) -> Result<()> {
    let word = &opts.word;
    match (&opts.relative_path, &opts.extension, &opts.project) {
        (None, None, false) => storage_backend.ignore(word),
        (None, Some(e), _) => storage_backend
            .ignore_store_mut()
            .ignore_for_extension(word, e),
        (Some(relative_path), None, _) => {
            let relative_path = project.get_relative_path(relative_path)?;
            storage_backend
                .ignore_store_mut()
                .ignore_for_path(word, project.id(), &relative_path)
        }
        (None, None, true) => storage_backend
            .ignore_store_mut()
            .ignore_for_project(word, project.id()),
        (Some(_), Some(_), _) => bail!("Cannot use both --relative-path and --extension"),
    }
}

fn remove(project: Project, mut storage_backend: StorageBackend, opts: &RemoveOpts) -> Result<()> {
    let word = &opts.word;
    match (&opts.relative_path, &opts.extension, &opts.project) {
        (None, None, false) => storage_backend.remove_ignored(word),
        (None, Some(e), _) => storage_backend.remove_ignored_for_extension(word, e),
        (Some(relative_path), None, _) => {
            let relative_path = project.get_relative_path(relative_path)?;
            storage_backend.remove_ignored_for_path(word, project.id(), &relative_path)
        }
        (None, None, true) => storage_backend
            .ignore_store_mut()
            .remove_ignored_for_project(word, project.id()),
        (Some(_), Some(_), _) => bail!("Cannot use both --relative-path and --extension"),
    }
}

fn check(
    project: Project,
    storage_backend: StorageBackend,
    dictionary: impl Dictionary,
    opts: &CheckOpts,
) -> Result<()> {
    info_1!(
        "Checking project {} for spelling errors",
        project.path().as_str().bold()
    );

    let interactive = !opts.non_interactive;

    match interactive {
        false => {
            let mut checker = NonInteractiveChecker::new(project, dictionary, storage_backend)?;
            check_with(&mut checker)
        }
        true => {
            let interactor = ConsoleInteractor;
            let mut checker =
                InteractiveChecker::new(project, interactor, dictionary, storage_backend)?;
            check_with(&mut checker)
        }
    }
}

fn check_with<C>(checker: &mut C) -> Result<()>
where
    C: Checker<Context = (usize, usize)>,
{
    let project = checker.project();
    let walker = walk(project)?;
    let mut checked = 0;
    for dir_entry in walker {
        let dir_entry = dir_entry?;
        let file_type = dir_entry.file_type().expect("walker yielded stdin");
        if !file_type.is_file() {
            continue;
        }
        let path = dir_entry.path();
        let relative_path = checker.to_relative_path(path)?;
        let token_processor = TokenProcessor::new(path);
        token_processor.each_token(|word, line, column| {
            checker.handle_token(word, &relative_path, &(line, column))
        })?;
        checked += 1;
    }

    info_3!("Checked {checked} files");

    checker.success()
}

fn clean(mut storage_backend: StorageBackend) -> Result<()> {
    storage_backend.clean()
}

fn undo(mut storage_backend: StorageBackend) -> Result<()> {
    storage_backend.undo()
}

fn suggest(dictionary: impl Dictionary, opts: &SuggestOpts) -> Result<()> {
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

// NOTE: we use this function to test the cli using a FakeDictionary
fn run<D: Dictionary>(
    project: Project,
    opts: &Opts,
    dictionary: D,
    storage_backend: StorageBackend,
) -> Result<()> {
    match &opts.action {
        Action::Add(opts) => add(project, storage_backend, opts),
        Action::Remove(opts) => remove(project, storage_backend, opts),
        Action::Check(opts) => check(project, storage_backend, dictionary, opts),
        Action::Suggest(opts) => suggest(dictionary, opts),
        Action::Undo => undo(storage_backend),
        Action::Clean => clean(storage_backend),
    }
}
pub fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let lang = match &opts.lang {
        Some(s) => s,
        None => "en_US",
    };

    let project_path = match opts.project_path.clone() {
        Some(p) => p,
        None => std::env::current_dir().context("Could not get current working directory")?,
    };

    let ignore_path = project_path.join(SKYSPELL_IGNORE_FILE);
    let mut ignore_config = None;

    if ignore_path.exists() {
        let kdl = std::fs::read_to_string(&ignore_path)
            .with_context(|| format!("While reading {SKYSPELL_IGNORE_FILE}"))?;
        ignore_config = Some(IgnoreConfig::parse(Some(ignore_path), &kdl)?);
    }

    let dictionary = EnchantDictionary::new(lang)?;
    let current_provider = dictionary.provider();

    let provider_in_config = ignore_config.as_ref().and_then(|c| c.provider());
    if let Some(provider_in_config) = provider_in_config {
        if current_provider != provider_in_config {
            bail!("Using '{current_provider}' as provider but should be '{provider_in_config}'")
        }
    }

    let use_db = ignore_config.as_ref().map(|c| c.use_db()).unwrap_or(true);
    let mut storage_backend = if use_db {
        let db_path = match opts.db_path.as_ref() {
            Some(s) => Ok(s.to_string()),
            None => get_default_db_path(lang),
        }?;
        info_1!("Using {db_path} as storage");
        let repository = SQLRepository::new(&db_path)?;
        StorageBackend::Repository(Box::new(repository))
    } else {
        let ignore_config =
            ignore_config.expect("ignore_config should not be None when use_db is false");
        info_1!("Using {SKYSPELL_IGNORE_FILE} as storage");
        StorageBackend::IgnoreStore(Box::new(ignore_config))
    };

    let project_path = ProjectPath::new(&project_path)?;
    let project = storage_backend.ensure_project(&project_path)?;

    let outcome = run(project, &opts, dictionary, storage_backend);
    if let Err(e) = outcome {
        print_error!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
