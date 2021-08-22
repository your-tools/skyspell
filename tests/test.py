import subprocess
import sqlite3

import pytest
import shutil

KITTY_WINDOW_TITLE = "skyspell-tests"


def send_keys(text):
    cmd = ["kitty", "@", "send-text", "--match", f"title:{KITTY_WINDOW_TITLE}", text]
    subprocess.run(cmd, check=True)


def run_query(tmp_path, sql):
    db_path = tmp_path / "tests.db"
    connection = sqlite3.connect(db_path)
    cursor = connection.cursor()
    cursor.execute(sql)
    rows = cursor.fetchall()
    return rows


def get_selection(tmp_path):
    send_keys(r":nop %sh{ echo $kak_selection > selection } \n")
    return (tmp_path / "selection").read_text().strip()


def get_option(tmp_path, option):
    send_keys(rf":nop %sh[ echo $kak_opt_{option} > option ] \n")
    return (tmp_path / "option").read_text().strip()


@pytest.fixture
def kitty_window():
    subprocess.run(
        ["kitty", "@", "launch", "--keep-focus", "--title", KITTY_WINDOW_TITLE, "sh"],
        check=True,
        capture_output=True,  # don't print the window ID
    )
    yield

    subprocess.run(
        ["kitty", "@", "close-window", "--match", f"title:{KITTY_WINDOW_TITLE}"],
        check=False,
        capture_output=True,  # don't print an error message in case this fails
    )


def start_kakoune(tmp_path, file_path):
    # Make sure skyspell is in PATH
    send_keys(r"export PATH=$HOME/.cargo/bin:$PATH \n")
    send_keys(r"which skyspell \n")

    # Set db path
    db_path = tmp_path / "tests.db"
    send_keys(fr"cd {tmp_path} \n")

    # Open the given file_path
    send_keys(f"SKYSPELL_DB_PATH={db_path} kak -n {file_path} \n")
    send_keys(r":evaluate-commands %sh{ skyspell kak init } \n")
    send_keys(r":skyspell-enable en_US \n")


def test_no_spelling_errors(kitty_window, tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("this is fine\n")
    start_kakoune(tmp_path, test_path)

    assert get_option(tmp_path, "skyspell_error_count") == "0"


def test_jump_to_first_error(kitty_window, tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("there is a missstake here\nand an othhher one there")
    start_kakoune(tmp_path, test_path)

    send_keys(r":skyspell-list \n")
    send_keys(r"\n")

    assert get_selection(tmp_path) == "missstake"


def test_goto_next(kitty_window, tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("there is a missstake here\nand an othhher one there")
    start_kakoune(tmp_path, test_path)

    send_keys("22l")
    send_keys(r":skyspell-next \n")
    assert get_selection(tmp_path) == "othhher"


def test_goto_previous(kitty_window, tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("there is a missstake here\nand an othhher one there")
    start_kakoune(tmp_path, test_path)

    send_keys("22l")
    send_keys(r":skyspell-previous \n")
    assert get_selection(tmp_path) == "missstake"


def test_add_global(kitty_window, tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(r":skyspell-list\n")
    send_keys("a")
    send_keys(r":quit\n")

    assert run_query(tmp_path, "SELECT word FROM ignored") == [("skyspell",)]


def test_add_to_project(kitty_window, tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(r":skyspell-list\n")
    send_keys("p")
    send_keys(r":quit\n")

    assert run_query(tmp_path, "SELECT word FROM ignored_for_project") == [
        ("skyspell",)
    ]


def test_add_to_file(kitty_window, tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(r":skyspell-list\n")
    send_keys("f")
    send_keys(r":quit\n")

    assert run_query(tmp_path, "SELECT word, path FROM ignored_for_path") == [
        ("skyspell", "foo.txt")
    ]


def test_add_to_extension(kitty_window, tmp_path):
    test_path = tmp_path / "foo.rs"
    test_path.write_text("fn function(parameter: type) { body }")

    start_kakoune(tmp_path, test_path)
    send_keys(r":skyspell-list\n")
    send_keys("e")
    send_keys(r":quit\n")

    assert run_query(tmp_path, "SELECT word, extension FROM ignored_for_extension") == [
        ("fn", "rs")
    ]


def test_skip_file_path(kitty_window, tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(r":skyspell-list\n")
    send_keys("s")
    send_keys(r":quit\n")

    assert run_query(tmp_path, "SELECT path FROM skipped_paths") == [("foo.txt",)]


def test_skip_file_name(kitty_window, tmp_path):
    test_path = tmp_path / "foo.lock"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(r":skyspell-list\n")
    send_keys("n")
    send_keys(r":quit\n")

    assert run_query(tmp_path, "SELECT file_name FROM skipped_file_names") == [
        ("foo.lock",)
    ]


def test_replace_with_suggestion(kitty_window, tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("There is a missstake here")

    start_kakoune(tmp_path, test_path)
    send_keys(r":skyspell-next\n")
    send_keys(r":skyspell-replace\n")
    send_keys(r"\n")  # select first menu entry
    send_keys(r":write-quit\n")

    actual = test_path.read_text()
    assert actual == "There is a mistake here\n"
