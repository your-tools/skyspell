use std::path::PathBuf;

use anyhow::Ok;
use anyhow::{Context, Result, bail};
use clap::Parser;
use colored::*;

use skyspell_core::Checker;
use skyspell_core::CheckerState;
use skyspell_core::Dictionary;
use skyspell_core::IgnoreStore;
use skyspell_core::Operation;
use skyspell_core::ProcessOutcome;
use skyspell_core::Project;
use skyspell_core::SystemDictionary;

mod checkers;
pub mod interactor;
pub use checkers::{InteractiveChecker, JsonChecker, NonInteractiveChecker};
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
    pub lang: String,

    #[clap(long, help = "Project path")]
    project_path: Option<PathBuf>,

    #[clap(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    #[clap(about = "Add word to one of the ignore lists")]
    Add(OperationOpts),
    #[clap(about = "Remove word from one of the ignore lists")]
    Remove(OperationOpts),
    #[clap(about = "Check files for spelling errors")]
    Check(CheckOpts),
    #[clap(about = "Suggest replacements for the given error")]
    Suggest(SuggestOpts),
    #[clap(about = "Undo last operation")]
    Undo,
}

#[derive(Parser)]
struct OperationOpts {
    #[clap(help = "The word to add/remove")]
    word: String,

    #[clap(long, help = "for the current project")]
    project: bool,

    #[clap(long, help = "for the given lang")]
    lang: Option<String>,

    #[clap(long, help = "for the given extension")]
    extension: Option<String>,

    #[clap(long, help = "for the given path")]
    relative_path: Option<PathBuf>,
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
pub struct CheckOpts {
    #[clap(
        long,
        help = "Don't ask what to do for each unknown word, instead just print the whole list - useful for continuous integration and other scripts"
    )]
    non_interactive: bool,

    #[clap(
        long,
        value_enum,
        help = "Output format: json implies --non-interactive"
    )]
    output_format: Option<OutputFormat>,

    #[clap(help = "List of paths to check")]
    paths: Vec<PathBuf>,

    #[clap(long, help = "Include git commit message file")]
    include_git_edit_message: bool,
}

#[derive(Parser)]
struct SuggestOpts {
    word: String,
}

fn add(
    mut state: CheckerState,
    project: Project,
    mut ignore_store: IgnoreStore,
    opts: &OperationOpts,
) -> Result<()> {
    let word = &opts.word;
    let mut operation = get_operation(project, opts, word)?;
    operation.execute(&mut ignore_store)?;
    state.set_last_operation(operation)?;
    Ok(())
}

fn remove(project: Project, mut ignore_store: IgnoreStore, opts: &OperationOpts) -> Result<()> {
    let word = &opts.word;
    let mut operation = get_operation(project, opts, word)?;
    operation.undo(&mut ignore_store)
}

fn get_operation(
    project: Project,
    opts: &OperationOpts,
    word: &str,
) -> Result<Operation, anyhow::Error> {
    let operation = match (
        &opts.relative_path,
        &opts.extension,
        &opts.project,
        &opts.lang,
    ) {
        (None, None, false, None) => Operation::new_ignore(word),
        (Some(relative_path), None, false, None) => {
            let relative_path = project.new_project_file(relative_path)?;
            Operation::new_ignore_for_path(word, &relative_path)
        }
        (None, Some(e), false, None) => Operation::new_ignore_for_extension(word, e),
        (None, None, true, None) => Operation::new_ignore_for_project(word),
        (None, None, false, Some(lang)) => Operation::new_ignore_for_lang(word, lang),
        _ => bail!("Conflicting options"),
    };
    Ok(operation)
}

fn check(
    project: Project,
    ignore_store: IgnoreStore,
    dictionary: impl Dictionary,
    opts: &CheckOpts,
) -> Result<()> {
    let output_format = opts.output_format.unwrap_or_default();
    let interactive = !opts.non_interactive && output_format != OutputFormat::Json;

    if interactive {
        let interactor = ConsoleInteractor;
        let mut checker =
            InteractiveChecker::new(project, interactor, dictionary, ignore_store, None)?;
        let _stats = check_with(&mut checker, opts)?;
        return checker.success();
    }

    match output_format {
        OutputFormat::Text => {
            let mut checker = NonInteractiveChecker::new(project, dictionary, ignore_store, opts)?;
            let stats = check_with(&mut checker, opts)?;
            let FileStats { skipped, checked } = stats;
            info_3!("Checked {checked} files - {skipped} skipped");
            checker.success()
        }
        OutputFormat::Json => {
            let mut checker = JsonChecker::new(project, dictionary, ignore_store)?;
            check_with(&mut checker, opts)?;
            checker.populate_result();
            let result = checker.result();
            let json = serde_json::to_string(result)?;
            println!("{json}");
            Ok(())
        }
    }
}

struct FileStats {
    skipped: usize,
    checked: usize,
}

fn check_with<C, D>(checker: &mut C, opts: &CheckOpts) -> Result<FileStats>
where
    C: Checker<D, SourceContext = ()>,
    D: Dictionary,
{
    let project = checker.project();
    let mut paths = opts.paths.clone();
    if paths.is_empty() {
        // No path provided on the command line, check the whole project
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
    if opts.include_git_edit_message {
        let git_message = project.path().join(".git/COMMIT_EDITMSG");
        if git_message.exists() {
            paths.push(git_message);
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

    Ok(FileStats { checked, skipped })
}

fn undo(mut state: CheckerState, mut ignore_store: IgnoreStore) -> Result<()> {
    let last_operation = state.pop_last_operation()?;
    let mut last_operation = match last_operation {
        None => bail!("Nothing to undo"),
        Some(o) => o,
    };
    last_operation.undo(&mut ignore_store)
}

fn suggest(dictionary: impl Dictionary, opts: &SuggestOpts) -> Result<()> {
    let word = &opts.word;
    if dictionary.check(word)? {
        return Ok(());
    }

    let suggestions = dictionary.suggest(word)?;

    for suggestion in suggestions.iter() {
        println!("{suggestion}");
    }

    Ok(())
}

fn run<D: Dictionary>(
    project: Project,
    opts: &Opts,
    dictionary: D,
    ignore_store: IgnoreStore,
    state: CheckerState,
) -> Result<()> {
    match &opts.action {
        Action::Add(opts) => add(state, project, ignore_store, opts),
        Action::Remove(opts) => remove(project, ignore_store, opts),
        Action::Check(opts) => check(project, ignore_store, dictionary, opts),
        Action::Suggest(opts) => suggest(dictionary, opts),
        Action::Undo => undo(state, ignore_store),
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
    let state = CheckerState::load(None)?;

    run(project, &opts, dictionary, ignore_store, state)
}

#[cfg(test)]
mod tests;
