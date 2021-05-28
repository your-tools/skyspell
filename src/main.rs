use std::convert::TryInto;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::Clap;
use platform_dirs::AppDirs;

use kak_spell::EnchantDictionary;
use kak_spell::{Checker, InteractiveChecker, KakouneChecker, NonInteractiveChecker};
use kak_spell::{ConsoleInteractor, Dictionary, Repo};

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
    #[clap(about = "Update the skipped lists")]
    Skip(SkipOpts),
    #[clap(about = "Used for the kak integration")]
    KakHook(KakHookOpts),
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

    #[clap(long)]
    kakoune: bool,

    #[clap(about = "List of paths to check")]
    sources: Vec<PathBuf>,
}

#[derive(Clap)]
struct ImportPersonalDictOpts {
    #[clap(long)]
    personal_dict_path: PathBuf,
}

#[derive(Clap, Debug)]
struct KakHookOpts {
    args: Vec<String>,
}

#[derive(Clap)]
struct SkipOpts {
    #[clap(long)]
    #[clap(about = "File path to skip")]
    full_path: Option<PathBuf>,

    #[clap(long)]
    #[clap(about = "Filename to skip")]
    file_name: Option<String>,
}

#[derive(Clap)]
struct SuggestOpts {
    word: String,

    #[clap(long, about = "Used by kakoune")]
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

fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let lang = opts.lang.unwrap_or_else(|| "en_US".to_string());

    match opts.action {
        Action::Add(opts) => add(&lang, opts),
        Action::Remove(opts) => remove(&lang, opts),
        Action::Check(opts) => check(&lang, opts),
        Action::ImportPersonalDict(opts) => import_personal_dict(&lang, opts),
        Action::KakHook(opts) => kak_hook(opts),
        Action::Suggest(opts) => suggest(&lang, opts),
        Action::Skip(opts) => skip(&lang, opts),
    }
}

fn open_db(lang: &str) -> Result<kak_spell::Db> {
    let app_dirs = AppDirs::new(Some("kak-spell"), false).unwrap();
    let data_dir = app_dirs.data_dir;
    std::fs::create_dir_all(&data_dir)
        .with_context(|| format!("Could not create {}", data_dir.display()))?;

    let db_path = &data_dir.join(format!("{}.db", lang));
    let db_path = db_path
        .to_str()
        .ok_or_else(|| anyhow!("{} contains non-UTF-8 chars", db_path.display()))?;
    kak_spell::db::new(db_path)
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

    match (interactive, opts.kakoune) {
        (false, false) => {
            let mut checker = NonInteractiveChecker::new(dictionary, repo);
            check_with(&mut checker, opts)
        }
        (true, false) => {
            let interactor = ConsoleInteractor;
            let mut checker = InteractiveChecker::new(interactor, dictionary, repo);
            check_with(&mut checker, opts)
        }
        (_, true) => {
            let mut checker = KakouneChecker::new(dictionary, repo);
            check_with(&mut checker, opts)?;
            checker.emit_kak_code()
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
            let tokenizer = kak_spell::Tokenizer::new(&line);
            for (word, pos) in tokenizer {
                checker.handle_token(&source_path, (i + 1, pos), word)?;
            }
        }
    }

    if !checker.success() {
        std::process::exit(1);
    }

    if opts.kakoune {
        return Ok(());
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
            println!("buffer {}", path_str);
            println!("select {}", selection);
        }
        "add-global" => {
            if !db.is_ignored(word)? {
                db.add_ignored(word)?;
            }
            kak_recheck(path_str);
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
            kak_recheck(path_str);
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
            kak_recheck(path_str);
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
            kak_recheck(path_str);
            println!("echo 'will now skip file named: \"{}\"'", file_name);
        }
        "skip-file" => {
            db.skip_full_path(path_str)?;
            kak_recheck(path_str);
            println!("echo 'will now skip the file \"{}\"'", path_str);
        }
        x => println!("echo -markup {{red}} unknown action: {}", x),
    };
    Ok(())
}

fn kak_recheck(path: &str) {
    println!("edit -existing {}", path);
    println!("write");
    println!("edit *spelling*");
}
