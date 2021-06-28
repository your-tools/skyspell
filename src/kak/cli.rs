use std::path::Path;

use crate::kak::checker::open_repository;
use crate::kak::helpers::*;
use crate::kak::KakouneChecker;
use crate::Checker;
use crate::EnchantDictionary;
use crate::RelativePath;
use crate::TokenProcessor;
use crate::{Dictionary, Repository};
use anyhow::{anyhow, Context, Result};
use clap::Clap;
use dirs_next::home_dir;

// Warning: most of the things written to stdout while this code
// is called will be interpreted as a Kakoune command.

// Use the debug() function in crate::kak::helpers for instead of dbg! or println!

#[derive(Clap)]
pub(crate) struct Opts {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Clap)]
enum Action {
    #[clap(about = "Add selection to the global ignore list")]
    AddGlobal,
    #[clap(about = "Add selection to the ignore list for the given extension")]
    AddExtension,
    #[clap(about = "Add selection to the ignore list for the given file")]
    AddFile,
    #[clap(about = "Add selection to the ignore list for the given project")]
    AddProject,
    #[clap(about = "Spell check every open buffer that belongs to the current project")]
    Check(CheckOpts),

    #[clap(about = "Display a menu containing suggestions")]
    Suggest,
    #[clap(about = "Skip the file name matching the selection")]
    SkipName,
    #[clap(about = "Skip the file path matching the selection")]
    SkipFile,

    #[clap(about = "Dump initial kakoune script")]
    Init,

    #[clap(about = "Jump to the selected error")]
    Jump,
    #[clap(about = "Jump to the previous error")]
    PreviousError(MoveOpts),
    #[clap(about = "Jump to the next error")]
    NextError(MoveOpts),
}

#[derive(Clap)]
struct CheckOpts {
    buflist: Vec<String>,
}

#[derive(Clap)]
struct MoveOpts {
    range_spec: String,
}

pub(crate) fn run(opts: Opts) -> Result<()> {
    match opts.action {
        Action::AddExtension => add_extension(),
        Action::AddFile => add_file(),
        Action::AddGlobal => add_global(),
        Action::AddProject => add_project(),
        Action::Check(opts) => check(opts),
        Action::Jump => jump(),
        Action::NextError(opts) => goto_next_error(opts),
        Action::PreviousError(opts) => goto_previous_error(opts),
        Action::SkipFile => skip_file(),
        Action::SkipName => skip_name(),
        Action::Suggest => suggest(),
        Action::Init => init(),
    }
}

struct LineSelection {
    path: String,
    word: String,
    selection: String,
}

fn parse_line_selection() -> Result<LineSelection> {
    let line_selection = get_selection()?;
    let (path, rest) = line_selection
        .split_once(": ")
        .with_context(|| "line selection should contain :")?;
    let (selection, word) = rest
        .split_once(' ')
        .with_context(|| "expected at least two words after the path name in line selection")?;
    Ok(LineSelection {
        path: path.to_string(),
        word: word.to_string(),
        selection: selection.to_string(),
    })
}

fn add_extension() -> Result<()> {
    let LineSelection { path, word, .. } = &parse_line_selection()?;
    let (_, ext) = path
        .rsplit_once(".")
        .ok_or_else(|| anyhow!("File has no extension"))?;
    let mut repository = open_repository()?;
    repository.ignore_for_extension(word, ext)?;
    kak_recheck();
    println!(
        "echo '\"{}\" added to the ignore list for  extension: \"{}\"'",
        word, ext
    );
    Ok(())
}

fn add_file() -> Result<()> {
    let LineSelection { path, word, .. } = &parse_line_selection()?;
    let path = &Path::new(path);
    let project = get_project()?;
    let relative_path = RelativePath::new(&project, path)?;
    let mut repository = open_repository()?;
    repository.ignore_for_path(word, &project, &relative_path)?;
    kak_recheck();
    println!(
        "echo '\"{}\" added to the ignore list for file: \"{}\"'",
        word, relative_path
    );
    Ok(())
}

fn add_global() -> Result<()> {
    let LineSelection { word, .. } = &parse_line_selection()?;
    let mut repository = open_repository()?;
    repository.ignore(word)?;
    kak_recheck();
    println!("echo '\"{}\" added to global ignore list'", word);
    Ok(())
}

fn add_project() -> Result<()> {
    let LineSelection { word, .. } = &parse_line_selection()?;
    let project = get_project()?;
    let mut repository = open_repository()?;
    repository.ignore_for_project(word, &project)?;
    kak_recheck();
    println!(
        "echo '\"{}\" added to ignore list for the current project'",
        word
    );
    Ok(())
}

fn jump() -> Result<()> {
    let LineSelection {
        path, selection, ..
    } = &parse_line_selection()?;
    println!("edit {}", path);
    println!("select {}", selection);
    Ok(())
}

fn check(opts: CheckOpts) -> Result<()> {
    let lang = get_lang()?;
    let project = get_project()?;
    let mut broker = enchant::Broker::new();
    let dictionary = EnchantDictionary::new(&mut broker, &lang)?;

    // Note:
    // kak_buflist may:
    //  * contain special buffers, like *debug*
    //  * use ~ for home dir
    //  * need to be canonicalize
    let repository = open_repository()?;
    let home_dir = home_dir().ok_or_else(|| anyhow!("Could not get home directory"))?;
    let home_dir = home_dir
        .to_str()
        .ok_or_else(|| anyhow!("Non-UTF8 chars in home dir"))?;
    let mut checker = KakouneChecker::new(project, dictionary, repository)?;
    for bufname in &opts.buflist {
        if bufname.starts_with('*') && bufname.ends_with('*') {
            continue;
        }

        // cleanup any errors that may have been set during last run
        println!("unset-option buffer={} spell_errors", bufname);

        let full_path = bufname.replace("~", home_dir);
        let source_path = Path::new(&full_path);

        if !source_path.exists() {
            continue;
        }

        let relative_path = checker.to_relative_path(&source_path)?;

        if checker.should_skip(&relative_path)? {
            continue;
        }

        if relative_path.as_str().starts_with("..") {
            continue;
        }

        let token_processor = TokenProcessor::new(&source_path)?;
        token_processor.each_token(|word, line, column| {
            checker.handle_token(&word, &relative_path, &(bufname.to_string(), line, column))
        })?;
    }

    checker.emit_kak_code()
}

enum Direction {
    Forward,
    Backward,
}

fn goto_error(opts: MoveOpts, direction: Direction) -> Result<()> {
    let range_spec = opts.range_spec;
    let cursor = get_cursor()?;
    let ranges = parse_range_spec(&range_spec)?;
    let new_range = match direction {
        Direction::Forward => get_next_selection(cursor, &ranges),
        Direction::Backward => get_previous_selection(cursor, &ranges),
    };
    let (line, start, end) = match new_range {
        None => return Ok(()),
        Some(x) => x,
    };
    println!(
        "select {line}.{start},{line}.{end}",
        line = line,
        start = start,
        end = end
    );
    Ok(())
}

fn goto_next_error(opts: MoveOpts) -> Result<()> {
    goto_error(opts, Direction::Forward)
}

fn goto_previous_error(opts: MoveOpts) -> Result<()> {
    goto_error(opts, Direction::Backward)
}

fn init() -> Result<()> {
    println!("{}", include_str!("init.kak"));
    Ok(())
}

fn skip_file() -> Result<()> {
    let LineSelection { path, .. } = &parse_line_selection()?;
    // We know it's a full path thanks to handle_error in KakouneChecker
    let full_path = Path::new(path);
    let project = get_project()?;

    let relative_path = RelativePath::new(&project, &full_path)?;

    let mut repository = open_repository()?;
    repository.skip_path(&project, &relative_path)?;

    kak_recheck();
    println!("echo 'will now skip \"{}\"'", relative_path);
    Ok(())
}

fn skip_name() -> Result<()> {
    let LineSelection { path, .. } = &parse_line_selection()?;
    let path = Path::new(path);
    let file_name = path
        .file_name()
        .with_context(|| "no file name")?
        .to_string_lossy();

    let mut repository = open_repository()?;
    repository.skip_file_name(&file_name)?;

    kak_recheck();
    println!("echo 'will now skip file named: \"{}\"'", file_name);
    Ok(())
}

fn suggest() -> Result<()> {
    let lang = &get_lang()?;
    let word = &get_selection()?;
    let mut broker = enchant::Broker::new();
    let dictionary = EnchantDictionary::new(&mut broker, lang)?;
    if dictionary.check(word)? {
        return Ok(());
    }

    let suggestions = dictionary.suggest(word);

    print!("menu ");
    for suggestion in suggestions.iter() {
        print!("%{{{}}} ", suggestion);
        print!("%{{execute-keys -itersel %{{c{}<esc>be}} ", suggestion);
        print!(":write <ret> :skyspell-check <ret>}}");
        print!(" ");
    }

    Ok(())
}

fn kak_recheck() {
    println!("write-all");
    println!("skyspell-check");
    println!("skyspell-list");
}
