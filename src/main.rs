use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Clap;
use platform_dirs::AppDirs;

use rcspell::ConsoleInteractor;
use rcspell::{Checker, InteractiveChecker, NonInteractiveChecker, Repo};

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Clap)]
enum Action {
    Add(AddOpts),
    ImportWordList(ImportWordListOpts),
    ImportPersonalDict(ImportPersonalDictOpts),
    Check(CheckOpts),
    Skip(SkipOpts),
}

#[derive(Clap)]
struct AddOpts {
    word: String,
    #[clap(long)]
    ext: Option<String>,
    #[clap(long)]
    file: Option<PathBuf>,
}

#[derive(Clap)]
struct CheckOpts {
    #[clap(long)]
    non_interactive: bool,

    sources: Vec<PathBuf>,
}

#[derive(Clap)]
struct ImportWordListOpts {
    #[clap(long)]
    list_path: Option<PathBuf>,
}

#[derive(Clap)]
struct ImportPersonalDictOpts {
    #[clap(long)]
    personal_dict_path: PathBuf,
}

#[derive(Clap)]
struct SkipOpts {
    #[clap(long)]
    full_path: Option<PathBuf>,

    #[clap(long)]
    file_name: Option<String>,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.action {
        Action::Add(opts) => add(opts),
        Action::Check(opts) => check(opts),
        Action::ImportWordList(opts) => import_word_list(opts),
        Action::ImportPersonalDict(opts) => import_personal_dict(opts),
        Action::Skip(opts) => skip(opts),
    }
}

fn open_db() -> Result<rcspell::Db> {
    let app_dirs = AppDirs::new(Some("rcspell"), false).unwrap();
    let data_dir = app_dirs.data_dir;
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Could not create {}", data_dir.display()))?;

    let db_path = &data_dir.join("en.db");
    let db_path = db_path
        .to_str()
        .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", db_path.display()))?;
    rcspell::db::new(db_path)
}

fn add(opts: AddOpts) -> Result<()> {
    let word = &opts.word;
    let mut db = open_db()?;

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

fn check(opts: CheckOpts) -> Result<()> {
    let db = open_db()?;
    match opts.non_interactive {
        true => {
            let mut checker = NonInteractiveChecker::new(db);
            check_with(&mut checker, opts)
        }
        false => {
            let interactor = ConsoleInteractor;
            let mut checker = InteractiveChecker::new(interactor, db);
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
        let source_path = std::fs::canonicalize(path)?;
        if checker.is_skipped(&source_path)? {
            skipped += 1;
            continue;
        }

        let source = File::open(&source_path)?;
        let reader = BufReader::new(source);

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            let tokenizer = rcspell::Tokenizer::new(&line);
            for (word, pos) in tokenizer {
                checker.handle_token(&source_path, (i + 1, pos), word)?;
            }
        }
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

fn import_word_list(opts: ImportWordListOpts) -> Result<()> {
    let mut db = open_db()?;
    if opts.list_path.is_none() {
        let words: Vec<&str> = include_str!("words_en.txt")
            .split_ascii_whitespace()
            .collect();
        db.insert_good_words(&words)?;
    } else {
        unimplemented!();
    }

    Ok(())
}

fn import_personal_dict(opts: ImportPersonalDictOpts) -> Result<()> {
    let mut db = open_db()?;
    let dict = std::fs::read_to_string(&opts.personal_dict_path)?;
    let words: Vec<&str> = dict.split_ascii_whitespace().collect();
    db.insert_ignored_words(&words)?;

    Ok(())
}

fn skip(opts: SkipOpts) -> Result<()> {
    let mut db = open_db()?;
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
