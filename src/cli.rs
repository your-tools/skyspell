use std::path::PathBuf;

use anyhow::{bail, Result};
use clap::Clap;

use crate::kak;
use crate::kak::io::new_kakoune_io;
use crate::TokenProcessor;
use crate::{Checker, InteractiveChecker, NonInteractiveChecker};
use crate::{ConsoleInteractor, Dictionary, Repository};
use crate::{Project, RelativePath};

pub fn run<D: Dictionary, R: Repository>(opts: Opts, dictionary: D, repository: R) -> Result<()> {
    let kakoune_io = new_kakoune_io();
    match opts.action {
        Action::Add(opts) => add(repository, opts),
        Action::Remove(opts) => remove(repository, opts),
        Action::Check(opts) => check(repository, dictionary, opts),
        Action::ImportPersonalDict(opts) => import_personal_dict(repository, opts),
        Action::Suggest(opts) => suggest(dictionary, opts),
        Action::Skip(opts) => skip(repository, opts),
        Action::Unskip(opts) => unskip(repository, opts),
        Action::Kak(opts) => match kak::cli::run(repository, dictionary, kakoune_io, opts) {
            Ok(()) => Ok(()),
            Err(e) => {
                println!("echo -markup {{Error}}{}", e);
                Err(e)
            }
        },
    }
}

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
pub struct Opts {
    #[clap(
        long,
        about = "Language to use",
        long_about = "Language to use - must match an installed dictionary for one of Enchant's provider"
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
            let project = Project::new(&project_path)?;
            let relative_path = RelativePath::new(&project, &relative_path)?;
            repository.ignore_for_path(word, &project, &relative_path)
        }
        (Some(project_path), None, None) => {
            let project = Project::new(&project_path)?;
            repository.ignore_for_project(word, &project)
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
            let project = Project::new(&project_path)?;
            let relative_path = RelativePath::new(&project, &relative_path)?;
            repository.remove_ignored_for_path(word, &project, &relative_path)
        }
        (Some(project_path), None, None) => {
            let project = Project::new(&project_path)?;
            repository.remove_ignored_for_project(word, &project)
        }
        (None, Some(_), None) => bail!("Cannot use --relative-path without --project-path"),
        (Some(_), _, Some(_)) => bail!("--extension is incompatible with --project-path"),
    }
}

fn check(repository: impl Repository, dictionary: impl Dictionary, opts: CheckOpts) -> Result<()> {
    let project = Project::new(&opts.project_path)?;

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
    }

    checker.success()?;

    match skipped {
        1 => println!("Skipped one file"),
        x if x >= 2 => println!("Skipped {} files", x),
        _ => (),
    }

    Ok(())
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
            let project = Project::new(&project_path)?;
            let relative_path = RelativePath::new(&project, &relative_path)?;
            repository.skip_path(&project, &relative_path)
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
            let project = Project::new(&project_path)?;
            let relative_path = RelativePath::new(&project, &relative_path)?;
            repository.unskip_path(&project, &relative_path)
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
