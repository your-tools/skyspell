use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Clap;
use platform_dirs::AppDirs;

use rcspell::Repo;

#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"))]
struct Opts {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Clap)]
enum Action {
    ImportWordList(ImportWordList),
    ImportPersonalDict(ImportPersonalDict),
    Check(Check),
}

#[derive(Clap)]
struct Check {
    sources: Vec<PathBuf>,
}

#[derive(Clap)]
struct ImportWordList {
    #[clap(long)]
    list_path: Option<PathBuf>,
}

#[derive(Clap)]
struct ImportPersonalDict {
    #[clap(long)]
    personal_dict_path: PathBuf,
}

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();

    match opts.action {
        Action::Check(opts) => check(opts),
        Action::ImportWordList(opts) => import_word_list(opts),
        Action::ImportPersonalDict(opts) => import_personal_dict(opts),
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

fn check(opts: Check) -> Result<()> {
    let interactor = rcspell::ConsoleInteractor;
    let db = open_db()?;
    let mut checker = rcspell::Checker::new(interactor, db);

    if opts.sources.is_empty() {
        println!("No path given - nothing to do");
    }

    for path in &opts.sources {
        let source_path = std::fs::canonicalize(path)?;

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

    if checker.skipped() {
        std::process::exit(1);
    }

    Ok(())
}

fn import_word_list(opts: ImportWordList) -> Result<()> {
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

fn import_personal_dict(opts: ImportPersonalDict) -> Result<()> {
    let mut db = open_db()?;
    let dict = std::fs::read_to_string(&opts.personal_dict_path)?;
    let words: Vec<&str> = dict.split_ascii_whitespace().collect();
    db.insert_ignored_words(&words)?;

    Ok(())
}
