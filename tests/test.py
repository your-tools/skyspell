import re
import sqlite3
import subprocess
import sys
import time

import pytest


class KittyWindow:
    def __init__(self, title):
        self.title = title
        process = subprocess.run(
            ["kitty", "@", "launch", "--title", title, "sh"],
            check=True,
            text=True,
        )

    def send_text(self, text):
        cmd = ["kitty", "@", "send-text", "--match", f"title:{self.title}", text]
        subprocess.run(cmd, check=True)

    def get_text(self):
        cmd = ["kitty", "@", "get-text", "--match", f"title:{self.title}"]
        process = subprocess.run(cmd, check=True, capture_output=True, text=True)
        return process.stdout

    def close(self):
        subprocess.run(
            ["kitty", "@", "close-window", "--match", f"title:{self.title}"],
            check=False,
            capture_output=True,  # don't print an error message in case this fails
        )


class RemoteKakoune:
    def __init__(self, kitty_window):
        self.kitty_window = kitty_window
        self.kitty_window.send_text(r"kak -n\n")

    def send_keys(self, keys):
        self.kitty_window.send_text(keys)

    def send_command(self, command, *args):
        self.kitty_window.send_text(r"\x1b")
        text = rf": {command} {' '.join(args)} \n"
        print(text)
        self.kitty_window.send_text(text)

    def extract_from_info(self, prefix):
        text = self.kitty_window.get_text()
        lines = text.splitlines()
        matching_lines = [x for x in text.splitlines() if prefix in x]
        assert len(matching_lines) == 1
        line = matching_lines[0]
        start = line.find(prefix)
        trail = line[start + len(prefix) :]
        match = re.search("`(.*)`", trail)
        assert match
        return match.groups()[0]

    def get_option(self, option):
        prefix = f"{option} => "
        self.send_command("info", "-title", "tests", f'"{prefix}`%opt[{option}]`"')
        return self.extract_from_info(prefix)

    def get_selection(self):
        prefix = "selection => "
        self.send_command("info", "-title", "tests", f'"{prefix}`%val[selection]`"')
        return self.extract_from_info(prefix)


class KakChecker(RemoteKakoune):
    def __init__(self, tmp_path):
        super().__init__(kitty_window)


def run_query(tmp_path, sql):
    db_path = tmp_path / "tests.db"
    connection = sqlite3.connect(db_path)
    cursor = connection.cursor()
    cursor.execute(sql)
    rows = cursor.fetchall()
    return rows


@pytest.fixture
def kitty_window():
    res = KittyWindow(title="skyspell-tests")
    yield res
    res.close()


@pytest.fixture
def remote_kakoune(kitty_window):
    res = RemoteKakoune(kitty_window)
    yield res
    res.send_command("quit!")


@pytest.fixture
def kak_checker(tmp_path, kitty_window):
    # Make sure skyspell is in PATH
    kitty_window.send_text(r"export PATH=$HOME/.cargo/bin:$PATH\n")

    # Set db path
    db_path = tmp_path / "tests.db"
    kitty_window.send_text(fr"cd {tmp_path} \n")
    kitty_window.send_text(fr"export SKYSPELL_DB_PATH={db_path}\n")

    kakoune = RemoteKakoune(kitty_window)
    kakoune.send_command("evaluate-commands", "%sh{ skyspell kak init }")
    kakoune.send_command("skyspell-enable", "en_US")

    yield kakoune

    kakoune.send_command("quit!")


def ensure_file(kak_checker, name, text):
    kak_checker.send_command("edit", name)
    kak_checker.send_keys(rf"I{text}")
    kak_checker.send_command("write")


def test_no_spelling_errors(kak_checker):
    ensure_file(kak_checker, "foo.txt", "There is no mistake there\n")
    actual = kak_checker.get_option("skyspell_error_count")
    assert actual == "0"


def test_jump_to_first_error(kak_checker):
    ensure_file(
        kak_checker, "foo.txt", r"There is a missstake here\nand an othhher one there"
    )
    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys(r"\n")
    assert kak_checker.get_selection() == "missstake"


def test_goto_next(kak_checker):
    ensure_file(
        kak_checker, "foo.txt", r"There is a missstake here\nand an othhher one there"
    )
    kak_checker.send_keys("gg22l")
    kak_checker.send_command("skyspell-next")
    assert kak_checker.get_selection() == "othhher"


def test_goto_previous(kak_checker):
    ensure_file(
        kak_checker, "foo.txt", r"There is a missstake here\nand an othhher one there"
    )
    kak_checker.send_keys("gg22l")
    kak_checker.send_command("skyspell-previous")
    assert kak_checker.get_selection() == "missstake"


def test_add_global(tmp_path, kak_checker):
    ensure_file(kak_checker, "foo.txt", r"I'm testing skyspell here")

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("a")
    kak_checker.send_command("quit")
    assert run_query(tmp_path, "SELECT word FROM ignored") == [("skyspell",)]


def test_add_to_project(tmp_path, kak_checker):
    ensure_file(kak_checker, "foo.txt", r"I'm testing skyspell here")

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("p")
    kak_checker.send_command("quit")

    assert run_query(tmp_path, "SELECT word FROM ignored_for_project") == [
        ("skyspell",)
    ]


def test_add_to_file(tmp_path, kak_checker):
    ensure_file(kak_checker, "foo.txt", r"I'm testing skyspell here")

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("f")
    kak_checker.send_command("quit")

    assert run_query(tmp_path, "SELECT word, path FROM ignored_for_path") == [
        ("skyspell", "foo.txt")
    ]


def test_add_to_extension(tmp_path, kak_checker):
    ensure_file(kak_checker, "foo.rs", "fn function(parameter: type) { body }")

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("e")
    kak_checker.send_command("quit")

    assert run_query(tmp_path, "SELECT word, extension FROM ignored_for_extension") == [
        ("fn", "rs")
    ]


def test_skip_file_path(tmp_path, kak_checker):
    ensure_file(kak_checker, "foo.txt", r"I'm testing skyspell here")

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("s")
    kak_checker.send_command("quit")

    assert run_query(tmp_path, "SELECT path FROM skipped_paths") == [("foo.txt",)]


def test_skip_file_name(tmp_path, kak_checker):
    ensure_file(kak_checker, "foo.lock", "notaword=42")

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("n")
    kak_checker.send_command("quit")

    assert run_query(tmp_path, "SELECT file_name FROM skipped_file_names") == [
        ("foo.lock",)
    ]


def test_replace_with_suggestion(tmp_path, kak_checker):
    ensure_file(kak_checker, "foo.txt", "There is a missstake here")

    kak_checker.send_command("skyspell-next")
    kak_checker.send_command("skyspell-replace")
    kak_checker.send_keys(r"\n")  # select first menu entry
    kak_checker.send_command("write-quit\n")

    actual = (tmp_path / "foo.txt").read_text()
    assert actual == "There is a mistake here\n"
