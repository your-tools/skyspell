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
use skyspell_core::{ProjectPath, RelativePath, SKYSPELL_IGNORE_FILE};

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

pub fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let lang = match &opts.lang {
        Some(s) => s,
        None => "en_US",
    };

    let ignore_path = PathBuf::from(SKYSPELL_IGNORE_FILE);

    let kdl = std::fs::read_to_string(&ignore_path)
        .with_context(|| "While reading {SKYSPELL_IGNORE_FILE}")?;
    let ignore_config = IgnoreConfig::parse(Some(ignore_path), &kdl)?;

    let dictionary = EnchantDictionary::new(lang)?;
    let current_provider = dictionary.provider();
    let config_provider = ignore_config.provider();

    if let Some(config_provider) = config_provider {
        if current_provider != config_provider {
            bail!("Using '{current_provider}' as provider but should be '{config_provider}'")
        }
    }

    let storage_backend;

    if ignore_config.use_db() {
        let db_path = match opts.db_path.as_ref() {
            Some(s) => Ok(s.to_string()),
            None => get_default_db_path(lang),
        }?;
        info_1!("Using {db_path} as storage");
        let repository = SQLRepository::new(&db_path)?;
        storage_backend = StorageBackend::Repository(Box::new(repository));
    } else {
        info_1!("Using {SKYSPELL_IGNORE_FILE} as storage");
        storage_backend = StorageBackend::IgnoreStore(Box::new(ignore_config));
    }

    let outcome = run(opts, dictionary, storage_backend);
    if let Err(e) = outcome {
        print_error!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

// NOTE: we use this function to test the cli using a FakeDictionary
fn run<D: Dictionary>(opts: Opts, dictionary: D, storage_backend: StorageBackend) -> Result<()> {
    match opts.action {
        Action::Add(opts) => add(storage_backend, opts),
        Action::Remove(opts) => remove(storage_backend, opts),
        Action::Check(opts) => check(storage_backend, dictionary, opts),
        Action::Suggest(opts) => suggest(dictionary, opts),
        Action::Undo => undo(storage_backend),
        Action::Clean => clean(storage_backend),
    }
}

fn clean(mut storage_backend: StorageBackend) -> Result<()> {
    storage_backend.clean()
}

#[derive(Parser)]
#[clap(version)]
pub struct Opts {
    #[clap(long, help = "Language to use")]
    pub lang: Option<String>,

    #[clap(long, help = "Path of the ignore repository")]
    pub db_path: Option<String>,

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
    #[clap(long, help = "Project path")]
    project_path: Option<PathBuf>,

    word: String,

    #[clap(long, help = "Add word to the ignore list for the given extension")]
    extension: Option<String>,

    #[clap(long, help = "Add word to the ignore list for the given path")]
    relative_path: Option<PathBuf>,
}

#[derive(Parser)]
struct CheckOpts {
    #[clap(long, help = "Project path")]
    project_path: Option<PathBuf>,

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
    #[clap(long, help = "Project path")]
    project_path: Option<PathBuf>,

    #[clap(
        long,
        help = "Remove word from the ignore list for the given extension"
    )]
    extension: Option<String>,
    #[clap(long, help = "Remove word from the ignore list for the given path")]
    relative_path: Option<PathBuf>,

    word: String,
}

fn add(mut storage_backend: StorageBackend, opts: AddOpts) -> Result<()> {
    let word = &opts.word;
    match (opts.project_path, opts.relative_path, opts.extension) {
        (None, None, None) => storage_backend.ignore(word),
        (None, _, Some(e)) => storage_backend
            .ignore_store_mut()
            .ignore_for_extension(word, &e),
        (Some(project_path), Some(relative_path), None) => {
            let project_path = ProjectPath::new(&project_path)?;
            let project = storage_backend.ensure_project(&project_path)?;
            let relative_path = RelativePath::new(&project_path, &relative_path)?;
            storage_backend
                .ignore_store_mut()
                .ignore_for_path(word, project.id(), &relative_path)
        }
        (Some(project_path), None, None) => {
            let project_path = ProjectPath::new(&project_path)?;
            let project = storage_backend.ensure_project(&project_path)?;
            storage_backend
                .ignore_store_mut()
                .ignore_for_project(word, project.id())
        }
        (None, Some(_), None) => bail!("Cannot use --relative-path without --project-path"),
        (Some(_), _, Some(_)) => bail!("--extension is incompatible with --project-path"),
    }
}

fn remove(mut storage_backend: StorageBackend, opts: RemoveOpts) -> Result<()> {
    let word = &opts.word;
    match (opts.project_path, opts.relative_path, opts.extension) {
        (None, None, None) => storage_backend.ignore_store_mut().remove_ignored(word),
        (None, _, Some(e)) => storage_backend
            .ignore_store_mut()
            .remove_ignored_for_extension(word, &e),
        (Some(project_path), Some(relative_path), None) => {
            let project_path = ProjectPath::new(&project_path)?;
            let project = storage_backend.ensure_project(&project_path)?;
            let relative_path = RelativePath::new(&project_path, &relative_path)?;
            storage_backend.ignore_store_mut().remove_ignored_for_path(
                word,
                project.id(),
                &relative_path,
            )
        }
        (Some(project_path), None, None) => {
            let project_path = ProjectPath::new(&project_path)?;
            let project = storage_backend.ensure_project(&project_path)?;
            storage_backend
                .ignore_store_mut()
                .remove_ignored_for_project(word, project.id())
        }
        (None, Some(_), None) => bail!("Cannot use --relative-path without --project-path"),
        (Some(_), _, Some(_)) => bail!("--extension is incompatible with --project-path"),
    }
}

fn check(
    mut storage_backend: StorageBackend,
    dictionary: impl Dictionary,
    opts: CheckOpts,
) -> Result<()> {
    let project_path = match opts.project_path {
        Some(p) => p,
        None => std::env::current_dir().context("Could not get current working directory")?,
    };
    let project_path = ProjectPath::new(&project_path)?;
    info_1!(
        "Checking project {} for spelling errors",
        project_path.as_str().bold()
    );

    let interactive = !opts.non_interactive;
    let project = storage_backend.ensure_project(&project_path)?;

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

fn undo(mut storage_backend: StorageBackend) -> Result<()> {
    storage_backend.undo()
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
mod tests;
