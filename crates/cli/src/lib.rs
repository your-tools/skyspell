use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::Parser;
use colored::*;

use skyspell_core::Checker;
use skyspell_core::Dictionary;
use skyspell_core::EnchantDictionary;
use skyspell_core::Config;
use skyspell_core::SkipFile;
use skyspell_core::TokenProcessor;
use skyspell_core::{Project, ProjectPath, SKYSPELL_CONFIG_FILE};

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
    pub lang: Option<String>,

    #[clap(long, help = "Path of the ignore repository")]
    pub db_path: Option<String>,

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

fn add(project: Project, mut ignore_config: Config, opts: &AddOpts) -> Result<()> {
    let word = &opts.word;
    match (&opts.relative_path, &opts.extension, &opts.project) {
        (None, None, false) => ignore_config.ignore(word),
        (None, Some(e), _) => ignore_config.ignore_for_extension(word, e),
        (Some(relative_path), None, _) => {
            let relative_path = project.get_relative_path(relative_path)?;
            ignore_config.ignore_for_path(word, &relative_path)
        }
        (None, None, true) => ignore_config.ignore_for_project(word),
        (Some(_), Some(_), _) => bail!("Cannot use both --relative-path and --extension"),
    }
}

fn remove(project: Project, mut ignore_config: Config, opts: &RemoveOpts) -> Result<()> {
    let word = &opts.word;
    match (&opts.relative_path, &opts.extension, &opts.project) {
        (None, None, false) => ignore_config.remove_ignored(word),
        (None, Some(e), _) => ignore_config.remove_ignored_for_extension(word, e),
        (Some(relative_path), None, _) => {
            let relative_path = project.get_relative_path(relative_path)?;
            ignore_config.remove_ignored_for_path(word, &relative_path)
        }
        (None, None, true) => ignore_config.remove_ignored_for_project(word),
        (Some(_), Some(_), _) => bail!("Cannot use both --relative-path and --extension"),
    }
}

fn check(
    project: Project,
    ignore_config: Config,
    dictionary: impl Dictionary,
    opts: &CheckOpts,
    output_format: OutputFormat,
) -> Result<()> {
    let interactive = !opts.non_interactive;

    match interactive {
        false => {
            let mut checker =
                NonInteractiveChecker::new(project, dictionary, ignore_config, output_format)?;
            check_with(&mut checker, &opts.paths, output_format)
        }
        true => {
            let interactor = ConsoleInteractor;
            let mut checker =
                InteractiveChecker::new(project, interactor, dictionary, ignore_config)?;
            check_with(&mut checker, &opts.paths, output_format)
        }
    }
}

fn check_with<C>(checker: &mut C, paths: &[PathBuf], output_format: OutputFormat) -> Result<()>
where
    C: Checker<Context = (usize, usize)>,
{
    let project = checker.project();
    let skip_file = SkipFile::new(project)?;
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
        let relative_path = checker.to_relative_path(&path)?;
        if skip_file.is_skipped(&relative_path) {
            skipped += 1;
        } else {
            let token_processor = TokenProcessor::new(&path);
            token_processor.each_token(|word, line, column| {
                checker.handle_token(word, &relative_path, &(line, column))
            })?;
            checked += 1;
        }
    }

    if output_format.is_text() {
        info_3!("Checked {checked} files - {skipped} skipped");
    }

    checker.success()
}

fn undo(mut _ignore_config: Config) -> Result<()> {
    bail!("Undo not implemented")
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
    ignore_config: Config,
) -> Result<()> {
    let output_format = opts.output_format.unwrap_or_default();
    match &opts.action {
        Action::Add(opts) => add(project, ignore_config, opts),
        Action::Remove(opts) => remove(project, ignore_config, opts),
        Action::Check(opts) => check(project, ignore_config, dictionary, opts, output_format),
        Action::Suggest(opts) => suggest(dictionary, opts),
        Action::Undo => undo(ignore_config),
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

    let ignore_path = project_path.join(SKYSPELL_CONFIG_FILE);

    let ignore_config = Config::open(&ignore_path)?;

    let dictionary = EnchantDictionary::new(lang)?;
    let current_provider = dictionary.provider();

    let provider_in_config = ignore_config.provider();
    if let Some(provider_in_config) = provider_in_config {
        if current_provider != provider_in_config {
            bail!("Using '{current_provider}' as provider but should be '{provider_in_config}'")
        }
    }

    let project_path = ProjectPath::new(&project_path)?;
    let project = Project::new(project_path);

    let outcome = run(project, &opts, dictionary, ignore_config);
    if let Err(e) = outcome {
        print_error!("{}", e);
        std::process::exit(1);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
