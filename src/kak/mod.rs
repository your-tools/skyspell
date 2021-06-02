use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use clap::Clap;
use dirs_next::home_dir;
use itertools::Itertools;

use crate::checker::lookup_token;
use crate::Db;
use crate::EnchantDictionary;
use crate::TokenProcessor;
use crate::{Dictionary, Repo};

const KAK_SPELL_LANG_OPT: &str = "kak_spell_lang";

// Warning: most of the things written to stdout while this code
// is called will be interpreted as a Kakoune command.

// Use the debug() function for instead of dbg! or println!

pub fn run() -> Result<()> {
    let opts: Opts = Opts::parse();
    dispatch(opts)
}

#[derive(Clap)]
#[clap(name="kak-spell (kakoune helper)", version = env!("CARGO_PKG_VERSION"))]
struct Opts {
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
    #[clap(about = "Spell check every buffer")]
    Check(CheckOpts),

    #[clap(about = "Display a menu containing suggestions")]
    Suggest,
    #[clap(about = "Skip the file name matching the selection")]
    SkipName,
    #[clap(about = "Skip the file path matching the selection")]
    SkipFile,

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

fn dispatch(opts: Opts) -> Result<()> {
    match opts.action {
        Action::AddExtension => add_extension(),
        Action::AddFile => add_file(),
        Action::AddGlobal => add_global(),
        Action::Check(opts) => check(opts),
        Action::Jump => jump(),
        Action::NextError(opts) => goto_next_error(opts),
        Action::PreviousError(opts) => goto_previous_error(opts),
        Action::SkipFile => skip_file(),
        Action::SkipName => skip_name(),
        Action::Suggest => suggest(),
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
    let mut db = open_db()?;
    if !db.known_extension(ext)? {
        db.add_extension(ext)?;
    }
    db.add_ignored_for_extension(word, ext)?;
    kak_recheck();
    println!(
        "echo '\"{}\" added to the ignore list for  extension: \"{}\"'",
        word, ext
    );
    Ok(())
}

fn add_file() -> Result<()> {
    let LineSelection { path, word, .. } = &parse_line_selection()?;
    let mut db = open_db()?;
    if !db.known_file(path)? {
        db.add_file(path)?;
    }
    db.add_ignored_for_file(word, path)?;
    kak_recheck();
    println!(
        "echo '\"{}\" added to the ignore list for file: \"{}\"'",
        word, path
    );
    Ok(())
}

fn add_global() -> Result<()> {
    let LineSelection { word, .. } = &parse_line_selection()?;
    let mut db = open_db()?;
    if !db.is_ignored(word)? {
        db.add_ignored(word)?;
    }
    kak_recheck();
    println!("echo '\"{}\" added to global ignore list'", word);
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

#[allow(dead_code)]
fn debug(message: &str) {
    println!("echo -debug {}", message);
}

fn get_from_environ(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| anyhow!("{} not found in environment", key))
}

fn get_option(name: &str) -> Result<String> {
    std::env::var(format!("kak_opt_{}", name)).map_err(|_| anyhow!("{} option not defined", name))
}

fn parse_usize(v: &str) -> Result<usize> {
    v.parse()
        .map_err(|_| anyhow!("could not parse {} as a positive number"))
}

fn get_cursor() -> Result<(usize, usize)> {
    let line = get_from_environ("kak_cursor_line")?;
    let column = get_from_environ("kak_cursor_column")?;
    Ok((parse_usize(&line)?, parse_usize(&column)?))
}

fn get_selection() -> Result<String> {
    get_from_environ("kak_selection")
}

fn get_lang() -> Result<String> {
    get_option(KAK_SPELL_LANG_OPT)
}

fn goto_previous_buffer() {
    println!("execute-keys ga")
}

fn check(opts: CheckOpts) -> Result<()> {
    let lang = get_lang()?;
    let mut broker = enchant::Broker::new();
    let dictionary = EnchantDictionary::new(&mut broker, &lang)?;

    // Note:
    // kak_buflist may:
    //  * contain special buffers, like *debug*
    //  * use ~ for home dir
    //  * need to be canonicalize
    let repo = open_db()?;
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

        let source_path = std::fs::canonicalize(&source_path)?;
        if checker.is_skipped(&source_path)? {
            continue;
        }

        let token_processor = TokenProcessor::new(&source_path)?;
        token_processor.each_token(|word, line, column| {
            checker.handle_token(&source_path, &bufname, (line, column), word)
        })?;
    }

    checker.emit_kak_code()
}

fn open_db() -> Result<crate::Db> {
    let lang = get_lang()?;
    Db::open(&lang)
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

fn skip_file() -> Result<()> {
    let LineSelection { path, .. } = &parse_line_selection()?;
    let path = PathBuf::from(path);
    let file_name = path
        .file_name()
        .with_context(|| "no file name")?
        .to_str()
        .with_context(|| "not an utf-8 file name")?;

    let mut db = open_db()?;
    db.skip_file_name(file_name)?;

    kak_recheck();
    println!("echo 'will now skip files named: \"{}\"'", file_name);
    Ok(())
}

fn skip_name() -> Result<()> {
    let LineSelection { path, .. } = &parse_line_selection()?;
    let mut db = open_db()?;
    db.skip_full_path(path)?;

    kak_recheck();
    println!("echo 'will now skip the file: \"{}\"'", path);
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
        print!(":write <ret> :kak-spell <ret>}}");
        print!(" ");
    }

    Ok(())
}

fn kak_recheck() {
    println!("write-all");
    println!("kak-spell-check");
    println!("kak-spell-list");
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

pub(crate) struct Error {
    pos: (usize, usize),
    buffer: String,
    path: PathBuf,
    token: String,
}

pub struct KakouneChecker<D: Dictionary, R: Repo> {
    dictionary: D,
    repo: R,
    errors: Vec<Error>,
}

impl<D: Dictionary, R: Repo> KakouneChecker<D, R> {
    pub fn new(dictionary: D, repo: R) -> Self {
        Self {
            dictionary,
            repo,
            errors: vec![],
        }
    }

    pub fn is_skipped(&self, path: &Path) -> Result<bool> {
        self.repo.is_skipped(path)
    }

    pub fn handle_token(
        &mut self,
        path: &Path,
        buffer: &str,
        pos: (usize, usize),
        token: &str,
    ) -> Result<()> {
        let found = lookup_token(&self.dictionary, &self.repo, token, path)?;
        if !found {
            self.errors.push(Error {
                path: path.to_path_buf(),
                pos,
                buffer: buffer.to_string(),
                token: token.to_string(),
            });
        }
        Ok(())
    }

    fn write_code(&self, f: &mut impl Write) -> Result<()> {
        let kak_timestamp =
            std::env::var("kak_timestamp").map_err(|_| anyhow!("kak_timestamp is not defined"))?;

        let kak_timestamp = kak_timestamp
            .parse::<usize>()
            .map_err(|_| anyhow!("could not parse kak_timestamp has a positive integer"))?;

        write_spelling_buffer(f, &self.errors)?;
        goto_previous_buffer();
        write_ranges(f, kak_timestamp, &self.errors)?;
        write_status(f, &self.errors)?;

        Ok(())
    }

    pub fn emit_kak_code(&self) -> Result<()> {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        self.write_code(&mut handle)?;

        Ok(())
    }
}

fn write_status(f: &mut impl Write, errors: &[Error]) -> Result<()> {
    match errors.len() {
        0 => write!(f, "echo -markup {{green}} no spelling errors"),
        1 => write!(f, "echo -markup {{red}} 1 spelling error"),
        n => write!(f, "echo -markup {{red}} {} spelling errors", n),
    }?;
    Ok(())
}

fn write_spelling_buffer(f: &mut impl Write, errors: &[Error]) -> Result<()> {
    // Open buffer
    writeln!(f, "edit -scratch *spelling*")?;

    // Delete everything
    write!(f, r"execute-keys \% <ret> d ")?;

    // Insert all errors
    write!(f, "i %{{")?;

    for error in errors.iter() {
        write_error(f, error)?;
        write!(f, "<ret>")?;
    }
    write!(f, "}} ")?;

    // Back to top
    writeln!(f, "<esc> gg")?;
    Ok(())
}

fn write_error(f: &mut impl Write, error: &Error) -> Result<()> {
    let Error {
        pos, token, path, ..
    } = error;
    let (line, start) = pos;
    let end = start + token.len();
    write!(
        f,
        "{}: {}.{},{}.{} {}",
        path.display(),
        line,
        start + 1,
        line,
        end,
        token
    )?;
    Ok(())
}

fn write_ranges(f: &mut impl Write, timestamp: usize, errors: &[Error]) -> Result<()> {
    for (buffer, group) in &errors.iter().group_by(|e| &e.buffer) {
        write!(
            f,
            "set-option buffer={} spell_errors {} ",
            buffer, timestamp
        )?;
        for error in group {
            write_error_range(f, error)?;
            write!(f, "  ")?;
        }
        writeln!(f)?;
    }
    Ok(())
}

fn write_error_range(f: &mut impl Write, error: &Error) -> Result<()> {
    let Error { pos, token, .. } = error;
    let (line, start) = pos;
    write!(f, "{}.{}+{}|Error", line, start + 1, token.len())?;
    Ok(())
}

pub fn get_previous_selection(
    cursor: (usize, usize),
    ranges: &[(usize, usize, usize)],
) -> Option<&(usize, usize, usize)> {
    let (cursor_line, cursor_col) = cursor;
    for range in ranges.iter().rev() {
        let &(start_line, _start_col, end_col) = range;

        if start_line > cursor_line {
            continue;
        }

        if start_line == cursor_line && end_col >= cursor_col {
            continue;
        }
        return Some(range);
    }

    // If we reach there, return the last error (auto-wrap)
    ranges.iter().last()
}

pub fn get_next_selection(
    cursor: (usize, usize),
    ranges: &[(usize, usize, usize)],
) -> Option<&(usize, usize, usize)> {
    let (cursor_line, cursor_col) = cursor;
    for range in ranges.iter() {
        let &(start_line, _start_col, end_col) = range;

        if start_line < cursor_line {
            continue;
        }

        if start_line == cursor_line && end_col <= cursor_col {
            continue;
        }
        return Some(range);
    }

    // If we reach there, return the first error (auto-wrap)
    ranges.iter().next()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_insert_errors() {
        let error = Error {
            pos: (2, 4),
            buffer: "hello.js".to_string(),
            path: PathBuf::from("/path/to/hello.js"),
            token: "foo".to_string(),
        };

        let mut buff: Vec<u8> = vec![];
        write_spelling_buffer(&mut buff, &[error]).unwrap();
        let actual = std::str::from_utf8(&buff).unwrap();
        let expected = r#"edit -scratch *spelling*
execute-keys \% <ret> d i %{/path/to/hello.js: 2.5,2.7 foo<ret>} <esc> gg
"#;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_write_ranges() {
        let err1 = Error {
            pos: (2, 4),
            buffer: "foo.js".to_string(),
            path: PathBuf::from("/path/to/foo.js"),
            token: "foo".to_string(),
        };

        let err2 = Error {
            pos: (3, 6),
            buffer: "foo.js".to_string(),
            path: PathBuf::from("/path/to/foo.js"),
            token: "bar".to_string(),
        };

        let err3 = Error {
            pos: (1, 5),
            path: PathBuf::from("/path/to/foo.js"),
            buffer: "spam.js".to_string(),
            token: "baz".to_string(),
        };

        let mut buff: Vec<u8> = vec![];
        write_ranges(&mut buff, 42, &[err1, err2, err3]).unwrap();
        let actual = std::str::from_utf8(&buff).unwrap();
        dbg!(actual);
    }

    #[test]
    fn goto_next_no_errors() {
        let pos = (1, 21);
        let ranges = [(1, 12, 19), (2, 19, 27)];
        let actual = get_previous_selection(pos, &ranges).unwrap();
        assert_eq!(actual, &(1, 12, 19));
    }
}
