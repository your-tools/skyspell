use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use clap::Clap;
use dirs_next::home_dir;

use crate::kak::io::KakouneIO;
use crate::kak::KakouneChecker;
use crate::os_io::OperatingSystemIO;
use crate::repository::RepositoryHandler;
use crate::Checker;
use crate::ProjectPath;
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

    let as_str = kakoune_io.get_option("skyspell_project")?;
    let path = PathBuf::from(as_str);
    let project = ProjectPath::new(&path)?;
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

    fn print(&self, message: &str) {
        self.kakoune_io().print(message)
    }

    fn dictionary(&self) -> &dyn Dictionary {
        self.checker.dictionary()
    }

    fn repository_handler(&mut self) -> &mut RepositoryHandler<R> {
        &mut self.checker.repository_handler
    }

    #[cfg(test)]
    fn repository(&self) -> &dyn Repository {
        self.checker.repository()
    }

    fn add_extension(&mut self) -> Result<()> {
        let LineSelection { path, word, .. } = &self.parse_line_selection()?;
        let (_, ext) = path
            .rsplit_once(".")
            .ok_or_else(|| anyhow!("File has no extension"))?;
        self.repository_handler().ignore_for_extension(word, ext)?;
        self.recheck();
        self.print(&format!(
            "echo '\"{}\" added to the ignore list for  extension: \"{}\"'",
            word, ext,
        ));
        Ok(())
    }

    fn add_file(&mut self) -> Result<()> {
        let LineSelection { path, word, .. } = &self.parse_line_selection()?;
        let project = &self.checker.project().clone();
        let relative_path = project.as_relative_path(path)?;
        self.repository_handler()
            .ignore_for_path(word, project.id(), &relative_path)?;
        self.recheck();
        self.print(&format!(
            "echo '\"{}\" added to the ignore list for file: \"{}\"'",
            word, relative_path
        ));
        Ok(())
    }

    fn add_global(&mut self) -> Result<()> {
        let LineSelection { word, .. } = &self.parse_line_selection()?;
        self.repository_handler().ignore(word)?;
        self.recheck();
        self.print(&format!("echo '\"{}\" added to global ignore list'", word));
        Ok(())
    }

    fn add_project(&mut self) -> Result<()> {
        let LineSelection { word, .. } = &self.parse_line_selection()?;
        let project_id = self.checker.project().id();
        self.repository_handler()
            .ignore_for_project(word, project_id)?;
        self.recheck();
        self.print(&format!(
            "echo '\"{}\" added to ignore list for the current project'",
            word
        ));
        Ok(())
    }

    fn jump(&self) -> Result<()> {
        let LineSelection {
            path, selection, ..
        } = self.parse_line_selection()?;
        self.print(&format!("edit {}\n", path));
        self.print(&format!("select {}\n", selection));
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
            self.print(&format!("unset-option buffer={} spell_errors\n", bufname));

            let full_path = bufname.replace("~", home_dir);
            let source_path = Path::new(&full_path);

            if !source_path.exists() {
                continue;
            }

            let relative_path = self.checker.to_relative_path(source_path)?;

            if self.checker.should_skip(&relative_path)? {
                continue;
            }

            if relative_path.as_str().starts_with("..") {
                continue;
            }

            let token_processor = TokenProcessor::new(source_path);
            token_processor.each_token(|word, line, column| {
                self.checker.handle_token(
                    word,
                    &relative_path,
                    &(bufname.to_string(), line, column),
                )
            })?;
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
        self.print(&format!(
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
        let project = self.checker.project().clone();
        let relative_path = project.as_relative_path(path)?;

        self.repository_handler()
            .skip_path(project.id(), &relative_path)?;

        self.recheck();
        self.print(&format!("echo 'will now skip \"{}\"\n'", relative_path));
        Ok(())
    }

    fn skip_name(&mut self) -> Result<()> {
        let LineSelection { path, .. } = &self.parse_line_selection()?;
        let path = Path::new(path);
        let file_name = path
            .file_name()
            .with_context(|| "no file name")?
            .to_string_lossy();

        self.repository_handler().skip_file_name(&file_name)?;

        self.recheck();
        self.print(&format!(
            "echo 'will now skip file named: \"{}\"'",
            file_name
        ));
        Ok(())
    }

    fn suggest(&mut self) -> Result<()> {
        let selection = &self.kakoune_io().get_selection()?;
        if selection.trim().is_empty() {
            bail!("Selection is blank");
        }
        if self.dictionary().check(selection)? {
            bail!("Selection: `{}` is not an error", selection);
        }

        let suggestions = self.dictionary().suggest(selection);

        if suggestions.is_empty() {
            bail!("No suggestions found");
        }

        self.print("menu ");
        for suggestion in suggestions.iter() {
            self.print(&format!("%{{{}}} ", suggestion));
            self.print(&format!(
                "%{{execute-keys -itersel %{{c{}<esc>be}} ",
                suggestion
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

#[cfg(test)]
mod tests {

    use super::*;
    use crate::project::RelativePath;

    use crate::kak::checker::tests::new_fake_checker;
    use crate::tests::FakeDictionary;
    use crate::tests::FakeIO;
    use crate::tests::FakeRepository;

    use tempdir::TempDir;

    type FakeCli = KakCli<FakeDictionary, FakeRepository, FakeIO>;

    fn new_cli(temp_dir: &TempDir) -> FakeCli {
        let fake_checker = new_fake_checker(temp_dir);
        let mut res = KakCli::new(fake_checker);
        res.set_timestamp(42);
        res
    }

    impl FakeCli {
        fn get_output(self) -> String {
            self.checker.get_output()
        }

        fn set_selection(&mut self, selection: &str) {
            self.checker.kakoune_io.set_selection(selection)
        }

        fn set_timestamp(&mut self, timestamp: usize) {
            self.checker.kakoune_io.set_timestamp(timestamp)
        }

        fn set_cursor(&mut self, line: usize, column: usize) {
            self.checker.kakoune_io.set_cursor(line, column)
        }

        fn ensure_path(&self, path: &str) -> RelativePath {
            self.checker.ensure_path(path)
        }

        fn write_file(&self, path: &str, contents: &str) {
            let project_path = self.checker.project().path();
            let full_path = project_path.as_ref().join(path);
            std::fs::write(&full_path, contents).unwrap();
        }

        fn add_known(&mut self, word: &str) {
            self.checker.add_known(word);
        }

        fn add_suggestions(&mut self, word: &str, suggestions: &[String]) {
            self.checker.add_suggestions(word, suggestions);
        }
    }

    #[test]
    fn test_parse_line_selection() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        cli.ensure_path("foo.py");
        let full_path = format!("{}/foo.py", temp_dir.path().display());
        cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

        let actual = cli.parse_line_selection().unwrap();

        assert_eq!(
            actual,
            LineSelection {
                path: full_path,
                word: "foo".to_string(),
                selection: "1.3,1.5".to_string(),
            }
        );
    }

    #[test]
    fn test_recheck() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let cli = new_cli(&temp_dir);
        cli.recheck();
        assert_eq!(
            cli.get_output(),
            "\
write-all
skyspell-check
skyspell-list
"
        );
    }

    #[test]
    fn test_get_project() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let cli = new_cli(&temp_dir);
        let actual = cli.checker.project().path();
        assert_eq!(actual.as_ref(), temp_dir.path());
    }

    #[test]
    fn test_add_extension() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        cli.ensure_path("foo.py");
        let full_path = format!("{}/foo.py", temp_dir.path().display());
        cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

        cli.add_extension().unwrap();

        assert!(cli
            .repository()
            .is_ignored_for_extension("foo", "py")
            .unwrap());
    }

    #[test]
    fn test_add_file() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        let foo_py = cli.ensure_path("foo.py");
        let full_path = format!("{}/foo.py", temp_dir.path().display());
        cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

        cli.add_file().unwrap();
        let project_id = cli.checker.project().id();

        assert!(cli
            .repository()
            .is_ignored_for_path("foo", project_id, &foo_py)
            .unwrap());
    }

    #[test]
    fn test_add_global() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        cli.ensure_path("foo.py");
        let full_path = format!("{}/foo.py", temp_dir.path().display());
        cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

        cli.add_global().unwrap();

        assert!(cli.repository().is_ignored("foo").unwrap());
    }

    #[test]
    fn test_add_project() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        let project_id = cli.checker.project().id();
        cli.ensure_path("foo.py");
        let full_path = format!("{}/foo.py", temp_dir.path().display());
        cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

        cli.add_project().unwrap();

        assert!(cli
            .repository()
            .is_ignored_for_project("foo", project_id)
            .unwrap());
    }

    #[test]
    fn test_jump() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        cli.ensure_path("foo.py");
        let full_path = format!("{}/foo.py", temp_dir.path().display());
        cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

        cli.jump().unwrap();

        let actual = cli.get_output();
        assert_eq!(
            actual,
            format!(
                "\
edit {}
select 1.3,1.5
",
                full_path
            )
        );
    }

    #[test]
    fn test_check_no_errors() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let project_path = temp_dir.path().to_string_lossy();
        let mut cli = new_cli(&temp_dir);
        cli.ensure_path("foo.py");
        let full_path = format!("{}/foo.py", temp_dir.path().display());

        let opts = CheckOpts {
            buflist: vec![full_path.clone()],
        };

        cli.check(&opts).unwrap();

        let actual = cli.get_output();
        assert_eq!(
            actual,
            format!(
                "\
unset-option buffer={full_path} spell_errors
edit -scratch *spelling*
execute-keys \\% <ret> d i %{{}} <esc> gg
execute-keys ga
set global skyspell_error_count 0
echo -markup {project_path}: {{green}}No spelling errors
",
                full_path = full_path,
                project_path = project_path
            )
        );
    }

    #[test]
    fn test_check_errors_in_two_buffers() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let project_path = temp_dir.path().to_string_lossy();
        let mut cli = new_cli(&temp_dir);
        cli.ensure_path("foo.md");
        cli.ensure_path("bar.md");
        cli.write_file("foo.md", "This is foo");
        cli.write_file("bar.md", "This is bar and it contains baz");
        for word in &["This", "is", "and", "it", "contains"] {
            cli.add_known(word);
        }
        let foo_path = format!("{}/foo.md", temp_dir.path().display());
        let bar_path = format!("{}/bar.md", temp_dir.path().display());

        let opts = CheckOpts {
            buflist: vec![foo_path.clone(), bar_path.clone()],
        };

        cli.check(&opts).unwrap();

        let actual = cli.get_output();
        let expected =
            format!(
                "\
unset-option buffer={foo_path} spell_errors
unset-option buffer={bar_path} spell_errors
edit -scratch *spelling*
execute-keys \\% <ret> d i %{{{foo_path}: 1.9,1.11 foo<ret>{bar_path}: 1.9,1.11 bar<ret>{bar_path}: 1.29,1.31 baz<ret>}} <esc> gg
execute-keys ga
set-option buffer={foo_path} spell_errors 42 1.9+3|Error \n\
set-option buffer={bar_path} spell_errors 42 1.9+3|Error 1.29+3|Error \n\
set global skyspell_error_count 3
echo -markup {project_path}: {{red}}3 spelling errors
",
                project_path = project_path,
                foo_path = foo_path,
                bar_path = bar_path,
            );
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_goto_next_error() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        let move_opts = MoveOpts {
            range_spec: "42 1.9,1.11|Error".to_string(),
        };

        cli.set_cursor(1, 2);
        cli.goto_next_error(move_opts).unwrap();

        assert_eq!(cli.get_output(), "select 1.9,1.11\n");
    }

    #[test]
    fn test_goto_previous_error() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        let move_opts = MoveOpts {
            range_spec: "42 1.9,1.11|Error".to_string(),
        };

        cli.set_cursor(1, 22);
        cli.goto_previous_error(move_opts).unwrap();

        assert_eq!(cli.get_output(), "select 1.9,1.11\n");
    }

    #[test]
    fn test_skip_file() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        let project_id = cli.checker.project().id();
        let foo_py = cli.ensure_path("foo.py");
        let foo_path = format!("{}/foo.py", temp_dir.path().display());
        cli.set_selection(&format!("{}: 1.3,1.5 foo", foo_path));

        cli.skip_file().unwrap();

        assert!(cli
            .repository()
            .is_skipped_path(project_id, &foo_py)
            .unwrap());
    }

    #[test]
    fn test_skip_name() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        cli.ensure_path("Cargo.lock");
        let lock_path = format!("{}/Cargo.lock", temp_dir.path().display());
        cli.set_selection(&format!("{}: 1.3,1.5 foo", lock_path));

        cli.skip_name().unwrap();

        assert!(cli.repository().is_skipped_file_name("Cargo.lock").unwrap());
    }

    #[test]
    fn test_suggest_on_error() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        cli.add_suggestions("hllo", &["hell".to_string(), "hello".to_string()]);
        cli.set_selection("hllo");

        cli.suggest().unwrap();

        let actual = cli.get_output();
        let expected = "\
menu \
%{hell} %{execute-keys -itersel %{chell<esc>be} :write <ret> :skyspell-check <ret>} \
%{hello} %{execute-keys -itersel %{chello<esc>be} :write <ret> :skyspell-check <ret>} ";

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_suggest_on_new_line_selection() {
        let temp_dir = TempDir::new("test-skyspell").unwrap();
        let mut cli = new_cli(&temp_dir);
        cli.set_selection("\n");

        let err = cli.suggest().unwrap_err();
        assert!(err.to_string().contains("blank"));
    }
}
