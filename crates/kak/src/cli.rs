use crate::{new_kakoune_io, KakouneChecker, KakouneIO};
use anyhow::{anyhow, Context, Result};
use clap::Parser;
use directories_next::BaseDirs;
use skyspell_core::Checker;
use skyspell_core::Dictionary;
use skyspell_core::OperatingSystemIO;
use skyspell_core::Operation;
use skyspell_core::Project;
use skyspell_core::SystemDictionary;
use std::path::{Path, PathBuf};

// Warning: most of the things written to stdout while this code is
// called will be interpreted as a Kakoune command. Use the debug()
// function in crate::kak::io for debugging instead of dbg! or println!

#[derive(Parser)]
#[clap(version)]
pub struct Opts {
    #[clap(long, help = "Language to use")]
    pub lang: Option<String>,

    #[clap(subcommand)]
    action: Action,
}

#[derive(Parser)]
enum Action {
    #[clap(about = "Dump initial kakoune script")]
    Init,

    #[clap(about = "Add selection to the global ignore list")]
    AddGlobal,
    #[clap(about = "Add selection to the ignore list for the given extension")]
    AddExtension,
    #[clap(about = "Add selection to the ignore list for the given file")]
    AddLang,
    #[clap(about = "Add selection to the ignore list for the current lang")]
    AddFile,
    #[clap(about = "Add selection to the ignore list for the given project")]
    AddProject,

    #[clap(about = "Spell check every open buffer that belongs to the current project")]
    Check(CheckOpts),
    #[clap(about = "Display a menu containing suggestions")]
    Suggest,

    #[clap(about = "Jump to the selected error")]
    Jump,
    #[clap(about = "Jump to the previous error")]
    PreviousError(MoveOpts),
    #[clap(about = "Jump to the next error")]
    NextError(MoveOpts),

    #[clap(about = "Undo last operation")]
    Undo,
}

#[derive(Parser)]
struct CheckOpts {
    buflist: Vec<String>,
}

#[derive(Parser)]
struct MoveOpts {
    range_spec: String,
}

#[derive(Debug, PartialEq, Eq)]
struct LineSelection {
    path: String,
    word: String,
    selection: String,
}

enum Direction {
    Forward,
    Backward,
}

pub fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    // Note: init is the only command that does not require a KakouneChecker
    if matches!(opts.action, Action::Init) {
        print!("{}", include_str!("init.kak"));
        return Ok(());
    }

    let kakoune_io = new_kakoune_io();

    let lang = &kakoune_io.get_option("skyspell_lang")?;
    let dictionary = SystemDictionary::new(lang)?;

    let project_path = kakoune_io.get_option("skyspell_project")?;
    let project_path = PathBuf::from(project_path);
    let project = Project::new(&project_path)?;
    let ignore_store = project.ignore_store()?;

    let checker = KakouneChecker::new(project, dictionary, ignore_store, kakoune_io, None)?;
    let mut cli = KakCli::new(checker)?;

    match opts.action {
        Action::AddExtension => cli.add_extension(),
        Action::AddLang => cli.add_lang(),
        Action::AddFile => cli.add_file(),
        Action::AddGlobal => cli.add_global(),
        Action::AddProject => cli.add_project(),
        Action::Check(opts) => cli.check(&opts),
        Action::Jump => cli.jump(),
        Action::NextError(opts) => cli.goto_next_error(opts),
        Action::PreviousError(opts) => cli.goto_previous_error(opts),
        Action::Suggest => cli.suggest(),
        Action::Undo => cli.checker.undo(),
        Action::Init => Ok(()), // handled above
    }
}

struct KakCli<D: Dictionary, S: OperatingSystemIO> {
    checker: KakouneChecker<D, S>,
    home_dir: String,
}

impl<D: Dictionary, S: OperatingSystemIO> KakCli<D, S> {
    fn new(checker: KakouneChecker<D, S>) -> Result<Self> {
        let base_dirs = BaseDirs::new().ok_or_else(|| anyhow!("Could not get home directory"))?;
        let home_dir = base_dirs
            .home_dir()
            .to_str()
            .ok_or_else(|| anyhow!("Non-UTF8 chars in home dir"))?;
        Ok(Self {
            home_dir: home_dir.to_string(),
            checker,
        })
    }

    fn kakoune_io(&self) -> &KakouneIO<S> {
        self.checker.io()
    }

    fn print(&self, message: &str) {
        self.kakoune_io().print(message)
    }

    fn print_error(&self, error: &str) {
        self.kakoune_io()
            .print(&format!("echo -markup {{Error}} {error}"))
    }

    fn dictionary(&self) -> &D {
        self.checker.dictionary()
    }

    fn add_extension(&mut self) -> Result<()> {
        let LineSelection { path, word, .. } = &self.parse_line_selection()?;
        let (_, ext) = path
            .rsplit_once('.')
            .ok_or_else(|| anyhow!("File has no extension"))?;
        let operation = Operation::new_ignore_for_extension(word, ext);
        self.checker.apply_operation(operation)?;
        self.recheck();
        self.print(&format!(
            "echo '\"{word}\" added to the ignore list for  extension: \"{ext}\"'"
        ));
        Ok(())
    }

    fn add_lang(&mut self) -> Result<()> {
        let lang = self.dictionary().lang().to_owned();
        let LineSelection { word, .. } = &self.parse_line_selection()?;
        let operation = Operation::new_ignore_for_lang(word, &lang);
        self.checker.apply_operation(operation)?;
        self.recheck();
        self.print(&format!(
            "echo '\"{word}\" added to the ignore list for \"{lang}\"'"
        ));
        Ok(())
    }

    fn add_file(&mut self) -> Result<()> {
        let LineSelection { path, word, .. } = &self.parse_line_selection()?;
        let project = &self.checker.project().clone();
        let relative_path = project.as_relative_path(path)?;
        let operation = Operation::new_ignore_for_path(word, &relative_path);
        self.checker.apply_operation(operation)?;
        self.recheck();
        self.print(&format!(
            "echo '\"{word}\" added to the ignore list for file: \"{relative_path}\"'"
        ));
        Ok(())
    }

    fn add_global(&mut self) -> Result<()> {
        let LineSelection { word, .. } = &self.parse_line_selection()?;
        let operation = Operation::new_ignore(word);
        self.checker.apply_operation(operation)?;
        self.recheck();
        self.print(&format!("echo '\"{word}\" added to global ignore list'"));
        Ok(())
    }

    fn add_project(&mut self) -> Result<()> {
        let LineSelection { word, .. } = &self.parse_line_selection()?;
        let operation = Operation::new_ignore_for_project(word);
        self.checker.apply_operation(operation)?;
        self.recheck();
        self.print(&format!(
            "echo '\"{word}\" added to ignore list for the current project'"
        ));
        Ok(())
    }

    fn jump(&self) -> Result<()> {
        let LineSelection {
            path, selection, ..
        } = self.parse_line_selection()?;
        self.print(&format!("edit {path}\n"));
        self.print(&format!("select {selection}\n"));
        Ok(())
    }

    fn unescape(&self, input: &str) -> String {
        match snailquote::unescape(input) {
            Err(_) => input.to_string(),
            Ok(s) => s,
        }
    }

    fn check(&mut self, opts: &CheckOpts) -> Result<()> {
        for bufname in &opts.buflist {
            // Note:
            // kak_buflist may:
            //  * be escaped
            //  * contain special buffers, like *debug*
            //  * use ~ for home dir
            let bufname = self.unescape(bufname);

            if bufname.starts_with('*') && bufname.ends_with('*') {
                // Probably a FIFO buffer, like *debug*, *grep* and the like
                continue;
            }

            // cleanup any errors that may have been set during last run
            self.print(&format!(
                "unset-option %{{buffer={bufname}}} skyspell_errors\n"
            ));

            let full_path = bufname.replace('~', &self.home_dir);
            let source_path = Path::new(&full_path);

            if !source_path.exists() {
                // Buffer has not been written to a file yet
                continue;
            }

            let relative_path = self.checker.to_relative_path(source_path)?;

            if relative_path.as_str().starts_with("..") {
                // Buffer is outside the current project
                continue;
            }

            self.checker.process(source_path, &bufname)?;
        }

        self.checker.write_code()
    }

    fn parse_line_selection(&self) -> Result<LineSelection> {
        let line_selection = self.kakoune_io().get_selection()?;
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

    fn goto_error(&self, opts: MoveOpts, direction: Direction) -> Result<()> {
        let range_spec = opts.range_spec;
        let cursor = self.kakoune_io().get_cursor()?;
        let ranges = self.kakoune_io().parse_range_spec(&range_spec)?;
        let new_range = match direction {
            Direction::Forward => self.kakoune_io().get_next_selection(cursor, &ranges),
            Direction::Backward => self.kakoune_io().get_previous_selection(cursor, &ranges),
        };
        let (line, start, end) = match new_range {
            None => return Ok(()),
            Some(x) => x,
        };
        self.print(&format!("select {line}.{start},{line}.{end}\n",));
        Ok(())
    }

    fn goto_next_error(&self, opts: MoveOpts) -> Result<()> {
        self.goto_error(opts, Direction::Forward)
    }

    fn goto_previous_error(&self, opts: MoveOpts) -> Result<()> {
        self.goto_error(opts, Direction::Backward)
    }

    fn suggest(&mut self) -> Result<()> {
        let selection = &self.kakoune_io().get_selection()?;
        if selection.trim().is_empty() {
            self.print_error("Selection is empty");
            return Ok(());
        }
        if self.dictionary().check(selection)? {
            self.print_error(&format!("Selection `{selection}` is not an error"));
            return Ok(());
        }

        let suggestions = self
            .dictionary()
            .suggest(selection)
            .context("While getting suggestions")?;

        if suggestions.is_empty() {
            self.print_error("No suggestions found");
            return Ok(());
        }

        self.print("menu ");
        for suggestion in suggestions.iter() {
            self.print(&format!("%{{{suggestion}}} "));
            self.print(&format!(
                "%{{execute-keys -itersel %{{c{suggestion}<esc>be}} ",
            ));
            self.print(":write <ret> :skyspell-check <ret>}");
            self.print(" ");
        }

        Ok(())
    }

    fn recheck(&self) {
        self.print("write-all\n");
        self.print("skyspell-check\n");
        self.print("skyspell-list\n");
    }
}
