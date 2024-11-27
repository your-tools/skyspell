use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::*;

use skyspell_core::Checker;
use skyspell_core::Dictionary;
use skyspell_core::IgnoreStore;
use skyspell_core::ProcessOutcome;
use skyspell_core::Project;
use skyspell_core::SystemDictionary;

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

#[derive(Debug, PartialEq, Eq, clap::ValueEnum, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Text,
    Json,
}

impl OutputFormat {
    fn is_text(&self) -> bool {
        matches!(self, OutputFormat::Text)
    }
}

#[derive(Parser)]
#[clap(version)]
pub struct Opts {
    #[clap(long, help = "Language to use")]
    pub lang: String,

    #[clap(long, help = "Project path")]
    project_path: Option<PathBuf>,

    #[clap(long, value_enum, short = 'o', help = "Output format")]
    output_format: Option<OutputFormat>,

    #[clap(subcommand)]
    action: Action,
}

impl Opts {
    pub fn text_output(&self) -> bool {
        self.output_format.unwrap_or_default() == OutputFormat::Text
    }
}

#[derive(Parser)]
enum Action {
    #[clap(about = "Add word to one of the ignore lists")]
    Add(AddOpts),
    #[clap(about = "Remove word from one of the ignore lists")]
    Remove(RemoveOpts),
    #[clap(about = "Check files for spelling errors")]
    Check(CheckOpts),
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

    #[clap(help = "List of paths to check")]
    paths: Vec<PathBuf>,
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

fn add(project: Project, mut ignore_store: IgnoreStore, opts: &AddOpts) -> Result<()> {
    let word = &opts.word;
    match (&opts.relative_path, &opts.extension, &opts.project) {
        (None, None, false) => ignore_store.ignore(word),
        (None, Some(e), _) => ignore_store.ignore_for_extension(word, e),
        (Some(relative_path), None, _) => {
            let relative_path = project.get_relative_path(relative_path)?;
            ignore_store.ignore_for_path(word, &relative_path)
        }
        (None, None, true) => ignore_store.ignore_for_project(word),
        (Some(_), Some(_), _) => bail!("Cannot use both --relative-path and --extension"),
    }
}

fn remove(project: Project, mut ignore_store: IgnoreStore, opts: &RemoveOpts) -> Result<()> {
    let word = &opts.word;
    match (&opts.relative_path, &opts.extension, &opts.project) {
        (None, None, false) => ignore_store.remove_ignored(word),
        (None, Some(e), _) => ignore_store.remove_ignored_for_extension(word, e),
        (Some(relative_path), None, _) => {
            let relative_path = project.get_relative_path(relative_path)?;
            ignore_store.remove_ignored_for_path(word, &relative_path)
        }
        (None, None, true) => ignore_store.remove_ignored_for_project(word),
        (Some(_), Some(_), _) => bail!("Cannot use both --relative-path and --extension"),
    }
}

fn check(
    project: Project,
    ignore_store: IgnoreStore,
    dictionary: impl Dictionary,
    opts: &CheckOpts,
    output_format: OutputFormat,
) -> Result<()> {
    let interactive = !opts.non_interactive;

    match interactive {
        false => {
            let mut checker =
                NonInteractiveChecker::new(project, dictionary, ignore_store, output_format)?;
            check_with(&mut checker, &opts.paths, output_format)
        }
        true => {
            let interactor = ConsoleInteractor;
            let mut checker =
                InteractiveChecker::new(project, interactor, dictionary, ignore_store, None)?;
            check_with(&mut checker, &opts.paths, output_format)
        }
    }
}

fn check_with<C, D>(checker: &mut C, paths: &[PathBuf], output_format: OutputFormat) -> Result<()>
where
    C: Checker<D, SourceContext = ()>,
    D: Dictionary,
{
    let project = checker.project();
    let mut paths = paths.to_vec();
    if paths.is_empty() {
        let walker = project.walk()?;
        for dir_entry in walker {
            let dir_entry = dir_entry?;
            let file_type = dir_entry.file_type().expect("walker yielded stdin");
            if !file_type.is_file() {
                continue;
            }
            let path = dir_entry.path();
            paths.push(path.to_path_buf());
        }
    }

    let mut checked = 0;
    let mut skipped = 0;
    for path in paths {
        let outcome = checker.process(&path, &())?;
        match outcome {
            ProcessOutcome::Skipped => skipped += 1,
            ProcessOutcome::Checked => checked += 1,
        }
    }

    if output_format.is_text() {
        info_3!("Checked {checked} files - {skipped} skipped");
    }

    checker.success()
}

fn undo(project: Project, dictionary: impl Dictionary, ignore_store: IgnoreStore) -> Result<()> {
    let interactor = ConsoleInteractor;
    let mut checker = InteractiveChecker::new(project, interactor, dictionary, ignore_store, None)?;
    checker.undo()
}

fn suggest(dictionary: impl Dictionary, opts: &SuggestOpts) -> Result<()> {
    let word = &opts.word;
    if dictionary.check(word)? {
        return Ok(());
    }

    let suggestions = dictionary.suggest(word)?;

    for suggestion in suggestions.iter() {
        println!("{}", suggestion);
    }

    Ok(())
}

fn run<D: Dictionary>(
    project: Project,
    opts: &Opts,
    dictionary: D,
    ignore_store: IgnoreStore,
) -> Result<()> {
    let output_format = opts.output_format.unwrap_or_default();
    match &opts.action {
        Action::Add(opts) => add(project, ignore_store, opts),
        Action::Remove(opts) => remove(project, ignore_store, opts),
        Action::Check(opts) => check(project, ignore_store, dictionary, opts, output_format),
        Action::Suggest(opts) => suggest(dictionary, opts),
        Action::Undo => undo(project, dictionary, ignore_store),
    }
}
pub fn main() -> Result<()> {
    SystemDictionary::init();

    let opts: Opts = Opts::parse();
    let lang = &opts.lang;
    let project_path = match opts.project_path.clone() {
        Some(p) => p,
        None => std::env::current_dir().context("Could not get current working directory")?,
    };

    let dictionary = SystemDictionary::new(lang)?;
    let project = Project::new(&project_path)?;
    let ignore_store = project.ignore_store()?;

    run(project, &opts, dictionary, ignore_store)
}

#[cfg(test)]
mod tests;
