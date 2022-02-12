use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Parser;
use colored::*;

use skyspell_core::repository::RepositoryHandler;
use skyspell_core::Checker;
use skyspell_core::EnchantDictionary;
use skyspell_core::TokenProcessor;
use skyspell_core::{get_default_db_path, SQLRepository};
use skyspell_core::{ConsoleInteractor, Dictionary, Repository};
use skyspell_core::{ProjectPath, RelativePath};

mod checkers;
pub use checkers::{InteractiveChecker, NonInteractiveChecker};

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
        Action::Skip(opts) => skip(repository, opts),
        Action::Unskip(opts) => unskip(repository, opts),
        Action::Undo => undo(repository),
        Action::Clean => clean(repository),
    }
}

fn clean(mut repository: SQLRepository) -> Result<()> {
    repository.clean()
}

#[derive(Parser)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
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
    #[clap(about = "Add path tho the given skipped list")]
    Skip(SkipOpts),
    #[clap(about = "Remove path from the given skipped list")]
    Unskip(UnskipOpts),
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
    project_path: PathBuf,

    #[clap(long)]
    non_interactive: bool,

    #[clap(help = "List of paths to check")]
    sources: Vec<PathBuf>,
}

#[derive(Parser)]
struct SkipOpts {
    #[clap(long, help = "Project path")]
    project_path: Option<PathBuf>,

    #[clap(long, help = "File path to skip")]
    relative_path: Option<PathBuf>,

    #[clap(long, help = "File name to skip")]
    file_name: Option<String>,
}

#[derive(Parser)]
struct UnskipOpts {
    #[clap(long, help = "Project path")]
    project_path: Option<PathBuf>,

    #[clap(long, help = "File path to unskip")]
    relative_path: Option<PathBuf>,

    #[clap(long, help = "File name to unskip")]
    file_name: Option<String>,
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
    let project_path = ProjectPath::new(&opts.project_path)?;
    info_1!(
        "Checking project {} for spelling errors",
        project_path.as_str().bold()
    );

    let interactive = !opts.non_interactive;
    let project = repository.ensure_project(&project_path)?;

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

        let token_processor = TokenProcessor::new(path);
        token_processor.each_token(|word, line, column| {
            checker.handle_token(word, &relative_path, &(line, column))
        })?;
        checked += 1;
    }

    info_3!("Checked {checked} files - {skipped} skipped");

    checker.success()
}

fn undo(repository: impl Repository) -> Result<()> {
    let mut handler = RepositoryHandler::new(repository);
    handler.undo()
}

fn skip(mut repository: impl Repository, opts: SkipOpts) -> Result<()> {
    match (opts.project_path, opts.relative_path, opts.file_name) {
        (Some(project_path), Some(relative_path), None) => {
            let project_path = ProjectPath::new(&project_path)?;
            let project = repository.ensure_project(&project_path)?;
            let relative_path = RelativePath::new(&project_path, &relative_path)?;
            repository.skip_path(project.id(), &relative_path)
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
            let project = ProjectPath::new(&project_path)?;
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
mod tests;
