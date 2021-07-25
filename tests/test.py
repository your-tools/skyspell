import subprocess
import sqlite3
import time

import pytest

TMUX_SESSION_NAME = "skyspell-tests"


def send_keys(*args, sleep=0):
    # Note: sometimes we need to wait for an filesystem operation. 300 ms
    # is good enough on my machine
    cmd = ["tmux", "send-keys", "-t", TMUX_SESSION_NAME, *args]
    subprocess.run(cmd, check=True)
    if sleep:
        time.sleep(sleep)


def run_query(tmp_path, sql):
    db_path = tmp_path / "tests.db"
    connection = sqlite3.connect(db_path)
    cursor = connection.cursor()
    cursor.execute(sql)
    rows = cursor.fetchall()
    return rows


def get_selection(tmp_path):
    send_keys(":nop %sh{ echo $kak_selection > selection }", "Enter", sleep=0.3)
    return (tmp_path / "selection").read_text().strip()


def get_option(tmp_path, option):
    send_keys(f":nop %sh[ echo $kak_opt_{option} > option ]", "Enter", sleep=0.5)
    return (tmp_path / "option").read_text().strip()


def start_kakoune(tmp_path, file_path):
    # Kill previous instance
    send_keys("Escape", sleep=0.3)  # because computers
    send_keys(":quit!", "Enter")

    # Set db path
    db_path = tmp_path / "tests.db"
    send_keys("cd", "Space", str(tmp_path), "Enter")
    # Open the given file_path
    send_keys(
        f"SKYSPELL_DB_PATH={db_path} kak -n {file_path}", "Enter", sleep=1
    )  # give kakone time to start

    # Setup skyspell
    send_keys(":evaluate-commands %sh{ skyspell kak init }", "Enter")
    send_keys(":skyspell-enable en_US", "Enter")


def test_no_spelling_errors(tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("this is fine\n")
    start_kakoune(tmp_path, test_path)

    assert get_option(tmp_path, "skyspell_error_count") == "0"


def test_jump_to_first_error(tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("there is a missstake here\nand an othhher one there")
    start_kakoune(tmp_path, test_path)

    send_keys(":skyspell-list", "Enter")
    send_keys("Enter")

    assert get_selection(tmp_path) == "missstake"


def test_goto_next(tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("there is a missstake here\nand an othhher one there")
    start_kakoune(tmp_path, test_path)

    send_keys("22l")
    send_keys(":skyspell-next", "Enter")
    assert get_selection(tmp_path) == "othhher"


def test_goto_previous(tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("there is a missstake here\nand an othhher one there")
    start_kakoune(tmp_path, test_path)

    send_keys("22l")
    send_keys(":skyspell-previous", "Enter")
    assert get_selection(tmp_path) == "missstake"


def test_add_global(tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(":skyspell-list", "Enter")
    send_keys("a", sleep=0.5)

    assert run_query(tmp_path, "SELECT word FROM ignored") == [("skyspell",)]


def test_add_to_project(tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(":skyspell-list", "Enter")
    send_keys("p", sleep=0.3)

    assert run_query(tmp_path, "SELECT word FROM ignored_for_project") == [
        ("skyspell",)
    ]


def test_add_to_file(tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(":skyspell-list", "Enter")
    send_keys("f", sleep=0.3)

    assert run_query(tmp_path, "SELECT word, path FROM ignored_for_path") == [
        ("skyspell", "foo.txt")
    ]


def test_add_to_extension(tmp_path):
    test_path = tmp_path / "foo.rs"
    test_path.write_text("fn function(parameter: type) { body }")

    start_kakoune(tmp_path, test_path)
    send_keys(":skyspell-list", "Enter")
    send_keys("e", sleep=0.3)

    assert run_query(tmp_path, "SELECT word, extension FROM ignored_for_extension") == [
        ("fn", "rs")
    ]


def test_skip_file_path(tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(":skyspell-list", "Enter")
    send_keys("s", sleep=0.3)

    assert run_query(tmp_path, "SELECT path FROM skipped_paths") == [("foo.txt",)]


def test_skip_file_name(tmp_path):
    test_path = tmp_path / "foo.lock"
    test_path.write_text("I'm testing skyspell here")

    start_kakoune(tmp_path, test_path)
    send_keys(":skyspell-list", "Enter")
    send_keys("n", sleep=0.3)

    assert run_query(tmp_path, "SELECT file_name FROM skipped_file_names") == [
        ("foo.lock",)
    ]


def test_replace_with_suggestion(tmp_path):
    test_path = tmp_path / "foo.txt"
    test_path.write_text("There is a missstake here")

    start_kakoune(tmp_path, test_path)
    send_keys(":skyspell-next", "Enter")
    send_keys(":skyspell-replace", "Enter")
    send_keys("Enter", sleep=0.3)  # select first menu entry
    send_keys(":write-quit", "Enter")

    actual = test_path.read_text()
    assert actual == "There is a mistake here\n"
