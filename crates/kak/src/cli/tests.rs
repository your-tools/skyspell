use super::*;

use skyspell_core::FakeDictionary;
use skyspell_core::FakeIO;
use skyspell_core::FakeRepository;
use skyspell_core::RelativePath;

use crate::checker::tests::new_fake_checker;

use tempfile::TempDir;

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
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
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
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
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
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let cli = new_cli(&temp_dir);
    let actual = cli.checker.project().path();
    assert_eq!(actual.as_ref(), temp_dir.path());
}

#[test]
fn test_add_extension() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut cli = new_cli(&temp_dir);
    cli.ensure_path("foo.py");
    let full_path = format!("{}/foo.py", temp_dir.path().display());
    cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

    cli.add_extension().unwrap();

    assert!(cli
        .ignore_store()
        .is_ignored_for_extension("foo", "py")
        .unwrap());
}

#[test]
fn test_add_file() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut cli = new_cli(&temp_dir);
    let foo_py = cli.ensure_path("foo.py");
    let full_path = format!("{}/foo.py", temp_dir.path().display());
    cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

    cli.add_file().unwrap();
    let project_id = cli.checker.project().id();

    assert!(cli
        .ignore_store()
        .is_ignored_for_path("foo", project_id, &foo_py)
        .unwrap());
}

#[test]
fn test_add_global() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut cli = new_cli(&temp_dir);
    cli.ensure_path("foo.py");
    let full_path = format!("{}/foo.py", temp_dir.path().display());
    cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

    cli.add_global().unwrap();

    assert!(cli.ignore_store().is_ignored("foo").unwrap());
}

#[test]
fn test_add_project() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut cli = new_cli(&temp_dir);
    let project_id = cli.checker.project().id();
    cli.ensure_path("foo.py");
    let full_path = format!("{}/foo.py", temp_dir.path().display());
    cli.set_selection(&format!("{}: 1.3,1.5 foo", full_path));

    cli.add_project().unwrap();

    assert!(cli
        .ignore_store()
        .is_ignored_for_project("foo", project_id)
        .unwrap());
}

#[test]
fn test_jump() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
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
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
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
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
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
set-option buffer={foo_path} spell_errors 42 1.9+3|SpellingError \n\
set-option buffer={bar_path} spell_errors 42 1.9+3|SpellingError 1.29+3|SpellingError \n\
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
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut cli = new_cli(&temp_dir);
    let move_opts = MoveOpts {
        range_spec: "42 1.9,1.11|SpellingError".to_string(),
    };

    cli.set_cursor(1, 2);
    cli.goto_next_error(move_opts).unwrap();

    assert_eq!(cli.get_output(), "select 1.9,1.11\n");
}

#[test]
fn test_goto_previous_error() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut cli = new_cli(&temp_dir);
    let move_opts = MoveOpts {
        range_spec: "42 1.9,1.11|SpellingError".to_string(),
    };

    cli.set_cursor(1, 22);
    cli.goto_previous_error(move_opts).unwrap();

    assert_eq!(cli.get_output(), "select 1.9,1.11\n");
}

#[test]
fn test_skip_file() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut cli = new_cli(&temp_dir);
    let project_id = cli.checker.project().id();
    let foo_py = cli.ensure_path("foo.py");
    let foo_path = format!("{}/foo.py", temp_dir.path().display());
    cli.set_selection(&format!("{}: 1.3,1.5 foo", foo_path));

    cli.skip_file().unwrap();

    assert!(cli
        .ignore_store()
        .is_skipped_path(project_id, &foo_py)
        .unwrap());
}

#[test]
fn test_skip_name() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut cli = new_cli(&temp_dir);
    cli.ensure_path("Cargo.lock");
    let lock_path = format!("{}/Cargo.lock", temp_dir.path().display());
    cli.set_selection(&format!("{}: 1.3,1.5 foo", lock_path));

    cli.skip_name().unwrap();

    assert!(cli
        .ignore_store()
        .is_skipped_file_name("Cargo.lock")
        .unwrap());
}

#[test]
fn test_suggest_on_error() {
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
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
    let temp_dir = tempfile::Builder::new()
        .prefix("test-skyspell")
        .tempdir()
        .unwrap();
    let mut cli = new_cli(&temp_dir);
    cli.set_selection("\n");

    let err = cli.suggest().unwrap_err();
    assert!(err.to_string().contains("blank"));
}
