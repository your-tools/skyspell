use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Clap;

use crate::kak;
use crate::Db;
use crate::EnchantDictionary;
use crate::TokenProcessor;
use crate::{Checker, InteractiveChecker, NonInteractiveChecker};
use crate::{ConsoleInteractor, Dictionary, Repository};

pub fn run() -> Result<()> {
    let opts: Opts = Opts::parse();
    let lang = opts.lang.unwrap_or_else(|| "en_US".to_string());

    match opts.action {
        Action::Add(opts) => add(&lang, opts),
        Action::Remove(opts) => remove(&lang, opts),
        Action::Check(opts) => check(&lang, opts),
        Action::ImportPersonalDict(opts) => import_personal_dict(&lang, opts),
        Action::Suggest(opts) => suggest(&lang, opts),
        Action::Skip(opts) => skip(&lang, opts),
        Action::Unskip(opts) => unskip(&lang, opts),
        Action::Kak(opts) => kak::cli::run(opts),
    }
}

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(
        long,
        about = "Language to use",
        long_about = "Language to use - must match an installed dictionary for one of Enchant's provider"
    )]
    lang: Option<String>,

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
    word: String,
    #[clap(long, about = "Add word to the ignore list for the given extension")]
    ext: Option<String>,
    #[clap(long, about = "Add word to the ignore list for the given path")]
    file: Option<PathBuf>,
}

#[derive(Clap)]
struct CheckOpts {
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
    #[clap(long, about = "File path to skip")]
    full_path: Option<PathBuf>,

    #[clap(long, about = "Filename to skip")]
    file_name: Option<String>,
}

#[derive(Clap)]
struct UnskipOpts {
    #[clap(long, about = "File path to unskip")]
    full_path: Option<PathBuf>,

    #[clap(long, about = "Filename to unskip")]
    file_name: Option<String>,
}

#[derive(Clap)]
struct SuggestOpts {
    word: String,
}

#[derive(Clap)]
struct RemoveOpts {
    word: String,
    #[clap(
        long,
        about = "Remove word from the ignore list for the given extension"
    )]
    ext: Option<String>,
    #[clap(long, about = "Remove word from the ignore list for the given path")]
    file: Option<PathBuf>,
}

fn open_db(lang: &str) -> Result<crate::Db> {
    Db::open(lang)
}

fn add(lang: &str, opts: AddOpts) -> Result<()> {
    let word = &opts.word;
    let mut db = open_db(lang)?;

    if let Some(p) = opts.file {
        let full_path = std::fs::canonicalize(p)?;
        let file = full_path
            .to_str()
            .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", full_path.display()))?;
        db.add_ignored_for_file(word, file)?;
    } else if let Some(e) = opts.ext {
        db.add_ignored_for_extension(word, &e)?;
    } else {
        db.add_ignored(word)?;
    }

    Ok(())
}

fn remove(lang: &str, opts: RemoveOpts) -> Result<()> {
    let word = &opts.word;
    let mut db = open_db(lang)?;

    if let Some(p) = opts.file {
        let full_path = std::fs::canonicalize(p)?;
        let file = full_path
            .to_str()
            .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", full_path.display()))?;
        db.remove_ignored_for_file(word, file)?;
    } else if let Some(e) = opts.ext {
        db.remove_ignored_for_extension(word, &e)?;
    } else {
        db.remove_ignored(word)?;
    }

    Ok(())
}

fn check(lang: &str, opts: CheckOpts) -> Result<()> {
    let mut broker = enchant::Broker::new();
    let dictionary = EnchantDictionary::new(&mut broker, lang)?;
    let repo = open_db(lang)?;
    let interactive = !opts.non_interactive;

    match interactive {
        false => {
            let mut checker = NonInteractiveChecker::new(dictionary, repo);
            check_with(&mut checker, opts)
        }
        true => {
            let interactor = ConsoleInteractor;
            let mut checker = InteractiveChecker::new(interactor, dictionary, repo);
            check_with(&mut checker, opts)
        }
    }
}

fn check_with<C: Checker>(checker: &mut C, opts: CheckOpts) -> Result<()> {
    let mut skipped = 0;
    if opts.sources.is_empty() {
        println!("No path given - nothing to do");
    }

    for path in &opts.sources {
        let source_path = std::fs::canonicalize(path)
            .with_context(|| format!("Could not canonicalize {}", path.display()))?;
        if checker.is_skipped(&source_path)? {
            skipped += 1;
            continue;
        }

        let token_processor = TokenProcessor::new(&source_path)?;
        token_processor.each_token(|word, line, column| {
            checker.handle_token(&source_path, (line, column), word)
        })?;
    }

    if !checker.success() {
        std::process::exit(1);
    }

    match skipped {
        1 => println!("Skipped one file"),
        x if x >= 2 => println!("Skipped {} files", x),
        _ => (),
    }

    Ok(())
}

fn import_personal_dict(lang: &str, opts: ImportPersonalDictOpts) -> Result<()> {
    let mut db = open_db(lang)?;
    let dict = std::fs::read_to_string(&opts.personal_dict_path)?;
    let words: Vec<&str> = dict.split_ascii_whitespace().collect();
    db.insert_ignored_words(&words)?;

    Ok(())
}

fn skip(lang: &str, opts: SkipOpts) -> Result<()> {
    let mut db = open_db(lang)?;
    if let Some(full_path) = opts.full_path {
        let full_path = std::fs::canonicalize(full_path)?;
        let full_path = full_path.to_str().with_context(|| "not valid utf-8")?;
        db.skip_full_path(full_path)?;
    }

    if let Some(file_name) = opts.file_name {
        db.skip_file_name(&file_name)?;
    }
    Ok(())
}

fn unskip(lang: &str, opts: UnskipOpts) -> Result<()> {
    let mut db = open_db(lang)?;
    if let Some(full_path) = opts.full_path {
        let full_path = std::fs::canonicalize(full_path)?;
        let full_path = full_path.to_str().with_context(|| "not valid utf-8")?;
        db.unskip_full_path(full_path)?;
    }

    if let Some(file_name) = opts.file_name {
        db.unskip_file_name(&file_name)?;
    }
    Ok(())
}

fn suggest(lang: &str, opts: SuggestOpts) -> Result<()> {
    let word = &opts.word;
    let mut broker = enchant::Broker::new();
    let dictionary = EnchantDictionary::new(&mut broker, lang)?;
    if dictionary.check(word)? {
        return Ok(());
    }

    let suggestions = dictionary.suggest(word);

    for suggestion in suggestions.iter() {
        println!("{}", suggestion);
    }

    Ok(())
}
