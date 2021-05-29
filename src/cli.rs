use std::convert::TryInto;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use clap::{AppSettings, Clap};
use dirs_next::home_dir;

use crate::Db;
use crate::EnchantDictionary;
use crate::Tokenizer;
use crate::{Checker, InteractiveChecker, KakouneChecker, NonInteractiveChecker};
use crate::{ConsoleInteractor, Dictionary, Repo};

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

        Action::KakCheck(opts) => kak_check(&lang, opts),
        Action::KakHook(opts) => kak_hook(opts),
        Action::Move(opts) => kak_move(opts),
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

    // Invoked by :kak-spell Kakoune command
    #[clap(setting=AppSettings::Hidden)]
    KakCheck(KakCheckOpts),

    #[clap(setting=AppSettings::Hidden)]
    // Invoked by Kakonue *spelling* hooks
    KakHook(KakHookOpts),

    #[clap(setting=AppSettings::Hidden)]
    // Invoked by :kak-spell-next, :kak-spell-previous
    Move(MoveOpts),
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

#[derive(Clap, Debug)]
struct KakCheckOpts {
    buflist: Vec<String>,
}

#[derive(Clap, Debug)]
struct KakHookOpts {
    #[clap(hidden = true)]
    args: Vec<String>,
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

    #[clap(long, hidden = true)]
    kakoune: bool,
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

#[derive(Clap)]
struct MoveOpts {
    args: Vec<String>,
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

fn kak_check(lang: &str, opts: KakCheckOpts) -> Result<()> {
    // Note:
    // kak_buflist may:
    //  * contain special buffers, like *debug*
    //  * use ~ for home dir
    //  * need to be canonicalize
    let mut broker = enchant::Broker::new();
    let dictionary = EnchantDictionary::new(&mut broker, lang)?;
    let repo = open_db(lang)?;
    let home_dir = home_dir().ok_or_else(|| anyhow!("Could not get home directory"))?;
    let home_dir = home_dir
        .to_str()
        .ok_or_else(|| anyhow!("Non-UTF8 chars in home dir"))?;
    let mut checker = KakouneChecker::new(dictionary, repo);
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

        let source_path = match std::fs::canonicalize(&source_path) {
            Err(e) => {
                // Should probably not happen, but the best we can do is write
                // to stderr ...
                // At least it will be visible in the *debug* buffer
                eprintln!("Could not canonicalize {} : {}", source_path.display(), e);
                continue;
            }
            Ok(p) => p,
        };

        if checker.is_skipped(&source_path)? {
            continue;
        }

        let source = match File::open(&source_path) {
            Ok(s) => s,
            Err(_) => {
                // Probably a buffer that has not been written to a file yet
                continue;
            }
        };
        let reader = BufReader::new(source);

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            let tokenizer = Tokenizer::new(&line);
            for (word, pos) in tokenizer {
                checker.handle_token(&source_path, &bufname, (i + 1, pos), word)?;
            }
        }
    }
    checker.emit_kak_code()
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

        let source = File::open(&source_path)
            .with_context(|| format!("Could not open {} for reading", source_path.display()))?;
        let reader = BufReader::new(source);

        for (i, line) in reader.lines().enumerate() {
            let line = line?;
            let tokenizer = Tokenizer::new(&line);
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

    if opts.kakoune {
        print!("menu ");
        for suggestion in suggestions.iter() {
            print!("%{{{}}} ", suggestion);
            print!("%{{execute-keys -itersel %{{c{}<esc>be}} ", suggestion);
            print!(":write <ret> :kak-spell <ret>}}");
            print!(" ");
        }
    } else {
        for suggestion in suggestions.iter() {
            println!("{}", suggestion);
        }
    }

    Ok(())
}

// Note: *anything* written to stdout while this code
// is called will be interpreted as a kakoune command
// Handle with care.
fn kak_hook(opts: KakHookOpts) -> Result<()> {
    // *spelling* buffer looks like this
    // path/to/foo.js: line.start,line.end word
    let args: [String; 3] = opts
        .args
        .try_into()
        .map_err(|_| anyhow!("Expected 2 arguments"))?;
    let [lang, action, selection] = &args;
    let mut db = open_db(lang)?;
    let (path_str, rest) = selection.split_once(": ").unwrap();
    let path = PathBuf::from(path_str);
    let (selection, word) = rest.split_once(' ').unwrap();
    match action.as_ref() {
        "jump" => {
            println!("edit {}", path_str);
            println!("select {}", selection);
        }
        "add-global" => {
            if !db.is_ignored(word)? {
                db.add_ignored(word)?;
            }
            kak_recheck();
            println!("echo '\"{}\" added to global ignore list'", word);
        }
        "add-extension" => {
            let (_, ext) = path_str
                .rsplit_once(".")
                .ok_or_else(|| anyhow!("File has no extension"))?;
            if !db.known_extension(ext)? {
                db.add_extension(ext)?;
            }
            db.add_ignored_for_extension(word, ext)?;
            kak_recheck();
            println!(
                "echo '\"{}\" added to the ignore list for  extension: \"{}\"'",
                word, ext
            );
        }
        "add-file" => {
            if !db.known_file(path_str)? {
                db.add_file(path_str)?;
            }
            db.add_ignored_for_file(word, path_str)?;
            kak_recheck();
            println!(
                "echo '\"{}\" added to the ignore list for file: \"{}\"'",
                word, path_str
            );
        }
        "skip-name" => {
            let file_name = path
                .file_name()
                .with_context(|| "no file name")?
                .to_str()
                .with_context(|| "not an utf-8 file name")?;
            db.skip_file_name(file_name)?;
            kak_recheck();
            println!("echo 'will now skip file named: \"{}\"'", file_name);
        }
        "skip-file" => {
            db.skip_full_path(path_str)?;
            kak_recheck();
            println!("echo 'will now skip the file \"{}\"'", path_str);
        }
        x => println!("echo -markup {{red}} unknown action: {}", x),
    };
    Ok(())
}

fn kak_recheck() {
    println!("write-all");
    println!("kak-spell");
    println!("buffer *spelling*");
}

fn parse_cursor(pos: &str) -> Result<(usize, usize)> {
    let (start, end) = pos.split_once('.').context("cursor should contain '.'")?;
    let start = start
        .parse::<usize>()
        .context("could not parse cursor start as an integer")?;
    let end = end
        .parse::<usize>()
        .context("could not parse cursor end as an integer")?;
    Ok((start, end))
}

fn parse_range_spec(range_spec: &str) -> Result<Vec<(usize, usize, usize)>> {
    // range-spec is empty
    if range_spec == "0" {
        return Ok(vec![]);
    }

    // Skip the timestamp
    let mut split = range_spec.split_whitespace();
    split.next();

    split.into_iter().map(|x| parse_range(x)).collect()
}

fn parse_range(range: &str) -> Result<(usize, usize, usize)> {
    let (range, _face) = range
        .split_once('|')
        .context("range spec should contain a face")?;
    let (start, end) = range
        .split_once(',')
        .context("range spec should contain ','")?;

    let (start_line, start_col) = parse_cursor(start)?;
    let (_end_line, end_col) = parse_cursor(end)?;

    Ok((start_line, start_col, end_col))
}

fn kak_move(opts: MoveOpts) -> Result<()> {
    let args: [String; 3] = opts
        .args
        .try_into()
        .map_err(|_| anyhow!("Expected 3 arguments"))?;
    let [direction, cursor, range_spec] = args;

    let cursor = parse_cursor(&cursor)?;
    let ranges = parse_range_spec(&range_spec)?;

    let new_range = match direction.as_ref() {
        "next" => crate::kak::get_next_selection(cursor, &ranges),
        "previous" => crate::kak::get_previous_selection(cursor, &ranges),
        _ => bail!("Unknown direction: {}", direction),
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
