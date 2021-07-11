use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use clap::Clap;
use dirs_next::home_dir;

use crate::kak::checker::SKYSPELL_PROJECT_OPT;
use crate::kak::io::{KakouneIO, OperatingSystemIO};
use crate::kak::KakouneChecker;
use crate::Checker;
use crate::Project;
use crate::RelativePath;
use crate::TokenProcessor;
use crate::{Dictionary, Repository};

// Warning: most of the things written to stdout while this code is
// called will be interpreted as a Kakoune command. Use the debug()
// function in crate::kak::io for debugging instead of dbg! or println!

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

struct LineSelection {
    path: String,
    word: String,
    selection: String,
}

enum Direction {
    Forward,
    Backward,
}

pub(crate) fn run<S: OperatingSystemIO>(
    repository: impl Repository,
    dictionary: impl Dictionary,
    kakoune_io: KakouneIO<S>,
    opts: Opts,
) -> Result<()> {
    // Note: init is the only command that does not require a KakouneChecker
    if matches!(opts.action, Action::Init) {
        print!("{}", include_str!("init.kak"));
        return Ok(());
    }

    let as_str = kakoune_io.get_option(SKYSPELL_PROJECT_OPT)?;
    let path = PathBuf::from(as_str);
    let project = Project::new(&path)?;
    let checker = KakouneChecker::new(project, dictionary, repository, kakoune_io)?;
    let mut cli = KakCli::new(checker);

    match opts.action {
        Action::AddExtension => cli.add_extension(),
        Action::AddFile => cli.add_file(),
        Action::AddGlobal => cli.add_global(),
        Action::AddProject => cli.add_project(),
        Action::Check(opts) => cli.check(&opts),
        Action::Jump => cli.jump(),
        Action::NextError(opts) => cli.goto_next_error(opts),
        Action::PreviousError(opts) => cli.goto_previous_error(opts),
        Action::SkipFile => cli.skip_file(),
        Action::SkipName => cli.skip_name(),
        Action::Suggest => cli.suggest(),
        _ => unreachable!(),
    }
}

struct KakCli<D: Dictionary, R: Repository, S: OperatingSystemIO> {
    checker: KakouneChecker<D, R, S>,
}

impl<D: Dictionary, R: Repository, S: OperatingSystemIO> KakCli<D, R, S> {
    fn new(checker: KakouneChecker<D, R, S>) -> Self {
        Self { checker }
    }

    fn kakoune_io(&self) -> &KakouneIO<S> {
        &self.checker.kakoune_io
    }

    fn repository(&mut self) -> &mut R {
        &mut self.checker.repository
    }

    fn dictionary(&mut self) -> &mut D {
        &mut self.checker.dictionary
    }

    fn get_project(&self) -> Result<Project> {
        let as_str = self.kakoune_io().get_option(SKYSPELL_PROJECT_OPT)?;
        let path = PathBuf::from(as_str);
        Project::new(&path)
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

    fn add_extension(&mut self) -> Result<()> {
        let LineSelection { path, word, .. } = &self.parse_line_selection()?;
        let (_, ext) = path
            .rsplit_once(".")
            .ok_or_else(|| anyhow!("File has no extension"))?;
        self.repository().ignore_for_extension(word, ext)?;
        self.kak_recheck();
        self.kakoune_io().print(&format!(
            "echo '\"{}\" added to the ignore list for  extension: \"{}\"'",
            word, ext,
        ));
        Ok(())
    }

    fn add_file(&mut self) -> Result<()> {
        let LineSelection { path, word, .. } = &self.parse_line_selection()?;
        let path = &Path::new(path);
        let project = self.get_project()?;
        let relative_path = RelativePath::new(&project, path)?;
        self.repository()
            .ignore_for_path(word, &project, &relative_path)?;
        self.kak_recheck();
        self.kakoune_io().print(&format!(
            "echo '\"{}\" added to the ignore list for file: \"{}\"'",
            word, relative_path
        ));
        Ok(())
    }

    fn add_global(&mut self) -> Result<()> {
        let LineSelection { word, .. } = &self.parse_line_selection()?;
        self.repository().ignore(word)?;
        self.kak_recheck();
        self.kakoune_io()
            .print(&format!("echo '\"{}\" added to global ignore list'", word));
        Ok(())
    }

    fn add_project(&mut self) -> Result<()> {
        let LineSelection { word, .. } = &self.parse_line_selection()?;
        let project = self.get_project()?;
        self.repository().ignore_for_project(word, &project)?;
        self.kak_recheck();
        self.kakoune_io().print(&format!(
            "echo '\"{}\" added to ignore list for the current project'",
            word
        ));
        Ok(())
    }

    fn jump(&self) -> Result<()> {
        let LineSelection {
            path, selection, ..
        } = self.parse_line_selection()?;
        self.kakoune_io().print(&format!("edit {}\n", path));
        self.kakoune_io().print(&format!("select {}\n", selection));
        Ok(())
    }

    fn check(&mut self, opts: &CheckOpts) -> Result<()> {
        // Note:
        // kak_buflist may:
        //  * contain special buffers, like *debug*
        //  * use ~ for home dir
        let home_dir = home_dir().ok_or_else(|| anyhow!("Could not get home directory"))?;
        let home_dir = home_dir
            .to_str()
            .ok_or_else(|| anyhow!("Non-UTF8 chars in home dir"))?;
        for bufname in &opts.buflist {
            if bufname.starts_with('*') && bufname.ends_with('*') {
                continue;
            }

            // cleanup any errors that may have been set during last run
            self.kakoune_io()
                .print(&format!("unset-option buffer={} spell_errors\n", bufname));

            let full_path = bufname.replace("~", home_dir);
            let source_path = Path::new(&full_path);

            if !source_path.exists() {
                continue;
            }

            let relative_path = self.checker.to_relative_path(&source_path)?;

            if self.checker.should_skip(&relative_path)? {
                continue;
            }

            if relative_path.as_str().starts_with("..") {
                continue;
            }

            let token_processor = TokenProcessor::new(&source_path)?;
            token_processor.each_token(|word, line, column| {
                self.checker.handle_token(
                    &word,
                    &relative_path,
                    &(bufname.to_string(), line, column),
                )
            })?;
        }

        self.checker.write_code()
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
        self.kakoune_io().print(&format!(
            "select {line}.{start},{line}.{end}\n",
            line = line,
            start = start,
            end = end
        ));
        Ok(())
    }

    fn goto_next_error(&self, opts: MoveOpts) -> Result<()> {
        self.goto_error(opts, Direction::Forward)
    }

    fn goto_previous_error(&self, opts: MoveOpts) -> Result<()> {
        self.goto_error(opts, Direction::Backward)
    }

    fn skip_file(&mut self) -> Result<()> {
        let LineSelection { path, .. } = &self.parse_line_selection()?;
        // We know it's a full path thanks to handle_error in KakouneChecker
        let full_path = Path::new(path);
        let project = self.get_project()?;

        let relative_path = RelativePath::new(&project, &full_path)?;

        self.repository().skip_path(&project, &relative_path)?;

        self.kak_recheck();
        println!("echo 'will now skip \"{}\"'", relative_path);
        Ok(())
    }

    fn skip_name(&mut self) -> Result<()> {
        let LineSelection { path, .. } = &self.parse_line_selection()?;
        let path = Path::new(path);
        let file_name = path
            .file_name()
            .with_context(|| "no file name")?
            .to_string_lossy();

        self.repository().skip_file_name(&file_name)?;

        self.kak_recheck();
        self.kakoune_io().print(&format!(
            "echo 'will now skip file named: \"{}\"'",
            file_name
        ));
        Ok(())
    }

    fn suggest(&mut self) -> Result<()> {
        let word = &self.kakoune_io().get_selection()?;
        if self.dictionary().check(word)? {
            bail!("Selection: `{}` is not an error", word);
        }

        let suggestions = self.dictionary().suggest(word);

        if suggestions.is_empty() {
            bail!("No suggestions found");
        }

        self.kakoune_io().print("menu ");
        for suggestion in suggestions.iter() {
            self.kakoune_io().print(&format!("%{{{}}} ", suggestion));
            self.kakoune_io().print(&format!(
                "%{{execute-keys -itersel %{{c{}<esc>be}} ",
                suggestion
            ));
            self.kakoune_io()
                .print(&format!(":write <ret> :skyspell-check <ret>}}"));
            self.kakoune_io().print(&format!(" "));
        }

        Ok(())
    }

    fn kak_recheck(&self) {
        self.kakoune_io().print("write-all\n");
        self.kakoune_io().print("skyspell-check\n");
        self.kakoune_io().print("skyspell-list\n");
    }
}
