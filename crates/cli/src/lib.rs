use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::*;

use skyspell_core::ignore_file::walk;
use skyspell_core::repository::RepositoryHandler;
use skyspell_core::Checker;
use skyspell_core::EnchantDictionary;
use skyspell_core::TokenProcessor;
use skyspell_core::{get_default_db_path, SQLRepository};
use skyspell_core::{Dictionary, Repository};
use skyspell_core::{ProjectPath, RelativePath};

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

    let db_path = match opts.db_path.as_ref() {
        Some(s) => Ok(s.to_string()),
        None => get_default_db_path(lang),
    }?;

    let repository = SQLRepository::new(&db_path)?;
    let dictionary = EnchantDictionary::new(lang)?;

    let outcome = run(opts, dictionary, repository);
    if let Err(e) = outcome {
        print_error!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

// NOTE: we use this function to test the cli using a FakeDictionary
fn run<D: Dictionary>(opts: Opts, dictionary: D, repository: SQLRepository) -> Result<()> {
    match opts.action {
        Action::Add(opts) => add(repository, opts),
        Action::Remove(opts) => remove(repository, opts),
        Action::Check(opts) => check(repository, dictionary, opts),
        Action::Suggest(opts) => suggest(dictionary, opts),
        Action::Undo => undo(repository),
        Action::Clean => clean(repository),
    }
}

fn clean(mut repository: SQLRepository) -> Result<()> {
    repository.clean()
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

    #[clap(long)]
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

fn add(mut repository: impl Repository, opts: AddOpts) -> Result<()> {
    let word = &opts.word;
    match (opts.project_path, opts.relative_path, opts.extension) {
        (None, None, None) => repository.ignore(word),
        (None, _, Some(e)) => repository.ignore_for_extension(word, &e),
        (Some(project_path), Some(relative_path), None) => {
            let project_path = ProjectPath::new(&project_path)?;
            let project = repository.ensure_project(&project_path)?;
            let relative_path = RelativePath::new(&project_path, &relative_path)?;
            repository.ignore_for_path(word, project.id(), &relative_path)
        }
        (Some(project_path), None, None) => {
            let project_path = ProjectPath::new(&project_path)?;
            let project = repository.ensure_project(&project_path)?;
            repository.ignore_for_project(word, project.id())
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
            let project_path = ProjectPath::new(&project_path)?;
            let project_id = repository.get_project_id(&project_path)?;
            let relative_path = RelativePath::new(&project_path, &relative_path)?;
            repository.remove_ignored_for_path(word, project_id, &relative_path)
        }
        (Some(project_path), None, None) => {
            let project_path = ProjectPath::new(&project_path)?;
            let project_id = repository.get_project_id(&project_path)?;
            repository.remove_ignored_for_project(word, project_id)
        }
        (None, Some(_), None) => bail!("Cannot use --relative-path without --project-path"),
        (Some(_), _, Some(_)) => bail!("--extension is incompatible with --project-path"),
    }
}

fn check(
    mut repository: impl Repository,
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
    let project = repository.ensure_project(&project_path)?;

    match interactive {
        false => {
            let mut checker = NonInteractiveChecker::new(project, dictionary, repository)?;
            check_with(&mut checker)
        }
        true => {
            let interactor = ConsoleInteractor;
            let mut checker = InteractiveChecker::new(project, interactor, dictionary, repository)?;
            check_with(&mut checker)
        }
    }
}

fn check_with<C>(checker: &mut C) -> Result<()>
where
    C: Checker<Context = (usize, usize)>,
{
    let project = checker.project();
    let walker = walk(project);
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

fn undo(repository: impl Repository) -> Result<()> {
    let mut handler = RepositoryHandler::new(repository);
    handler.undo()
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
