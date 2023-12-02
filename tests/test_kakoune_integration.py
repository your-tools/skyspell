import os
import kdl
import re
import subprocess
import time
from pathlib import Path
from typing import Iterator

import pytest

SOCKET_PATH = "unix:/tmp/kitty.sock"


class KittyWindow:
    def __init__(self, socket_path: Path) -> None:
        self.socket_path = socket_path
        env = os.environ.copy()
        env["PS1"] = r"\w$ "
        env["PATH"] = os.environ["PATH"]

        self.process = subprocess.Popen(
            [
                "kitty",
                "--config",
                "None",
                "--class",
                "kitty-tests",
                "-o",
                "allow_remote_control=yes",
                "-o",
                "enable_audio_bell=no",
                "--listen-on",
                f"unix:{self.socket_path}",
                "sh",
            ],
            env=env,
        )
        # Wait until the kitty server is done starting
        while not self.socket_path.exists():
            time.sleep(0.1)

    def send_text(self, text: str) -> None:
        cmd = [
            "kitty",
            "@",
            "--to",
            f"unix:{self.socket_path}",
            "send-text",
            text,
        ]
        subprocess.run(cmd, check=True)

    def get_text(self) -> str:
        cmd = [
            "kitty",
            "@",
            "--to",
            f"unix:{self.socket_path}",
            "get-text",
        ]
        process = subprocess.run(cmd, check=True, capture_output=True, text=True)
        return process.stdout

    def close(self) -> None:
        self.process.kill()


class RemoteKakoune:
    def __init__(self, kitty_window: KittyWindow) -> None:
        self.kitty_window = kitty_window
        self.kitty_window.send_text(r"kak -n\n")

    def send_keys(self, keys: str) -> None:
        human_readable = re.sub("\x1b(.)", r"alt-\1 ", keys)
        print(human_readable)
        self.kitty_window.send_text(keys)

    def send_command(self, command: str, *args: str) -> None:
        self.kitty_window.send_text(r"\x1b")
        text = rf":{command} {' '.join(args)} \n"
        self.kitty_window.send_text(text)
        human_readable = text.replace(r"\n", "\n")
        print(human_readable, end="")

    def extract_from_debug_buffer(self, prefix: str) -> str:
        self.send_command("edit", "-existing", "*debug*")
        text = self.kitty_window.get_text()
        matching_lines = [x for x in text.splitlines() if x.startswith(prefix)]
        # If the value has changed, we want the latest
        line = matching_lines[-1]
        res = line[len(prefix) :]
        self.send_command("buffer-previous")
        return res

    def get_option(self, option: str) -> str:
        prefix = f"option : {option} => "
        self.send_command("echo", "-debug", prefix, f"%opt[{option}]")
        return self.extract_from_debug_buffer(prefix)

    def get_selection(self) -> str:
        prefix = "selection => "
        self.send_command("echo", "-debug", prefix, "%val{selection}")
        return self.extract_from_debug_buffer(prefix)


def parse_config(tmp_path: Path) -> kdl.Document:
    config_path = tmp_path / "skyspell.kdl"
    return kdl.parse(config_path.read_text())


@pytest.fixture
def kitty_window(tmp_path: Path) -> Iterator[KittyWindow]:
    socket_path = tmp_path / "kitty.sock"
    res = KittyWindow(socket_path=socket_path)
    yield res
    res.close()


@pytest.fixture
def kak_checker(tmp_path: Path, kitty_window: KittyWindow) -> Iterator[RemoteKakoune]:
    db_path = tmp_path / "tests.db"
    kitty_window.send_text(rf"cd {tmp_path} \n")

    kakoune = RemoteKakoune(kitty_window)
    kakoune.send_command(
        "hook",
        "global",
        "RuntimeError",
        ".+",
        "%{echo -to-file err.txt %val{hook_param}}",
    )
    kakoune.send_command("evaluate-commands", "%sh{ skyspell-kak init }")
    kakoune.send_command("set-option", "global", "skyspell_db_path", str(db_path))
    kakoune.send_command("skyspell-enable", "en_US")

    yield kakoune

    kakoune.send_command("quit!")
    err_txt = tmp_path / "err.txt"
    if err_txt.exists():
        pytest.fail(
            f"Some kakoune errors occurred during the test:\n{err_txt.read_text()}"
        )


def open_file_with_contents(kak_checker: RemoteKakoune, path: Path, text: str) -> None:
    path.write_text(text)
    kak_checker.send_command(f"edit %{{{path}}}")
    kak_checker.send_command("skyspell-check")


def test_no_spelling_errors(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker, tmp_path / "foo.txt", "There is no mistake there\n"
    )
    actual = kak_checker.get_option("skyspell_error_count")
    assert actual == "0"


def test_jump_to_first_error(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker,
        tmp_path / "foo.txt",
        "There is a missstake here\nand an othhher one there",
    )
    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys(r"\n")
    assert kak_checker.get_selection() == "missstake"


def test_do_not_break_dot(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    original_contents = "There is a missstake here\n"
    foo_path = tmp_path / "foo.txt"
    open_file_with_contents(
        kak_checker,
        foo_path,
        original_contents,
    )
    kak_checker.send_command("skyspell-list")
    kak_checker.send_command(f"edit {foo_path}")
    kak_checker.send_keys(".")
    kak_checker.send_command("write")
    new_contents = foo_path.read_text()
    assert original_contents == new_contents


def test_goto_next(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker,
        tmp_path / "foo.txt",
        "There is a missstake here\nand an othhher one there",
    )
    kak_checker.send_keys("gg22l")
    kak_checker.send_command("skyspell-next")
    assert kak_checker.get_selection() == "othhher"


def test_goto_previous(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker,
        tmp_path / "foo.txt",
        r"There is a missstake here\nand an othhher one there",
    )
    kak_checker.send_keys("gg22l")
    kak_checker.send_command("skyspell-previous")
    assert kak_checker.get_selection() == "missstake"


def test_add_global(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker, tmp_path / "foo.txt", r"I'm testing skyspell here"
    )

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("a")
    kak_checker.send_command("quit")

    config = parse_config(tmp_path)
    global_words = [node.name for node in config['global'].nodes]
    assert "skyspell" in global_words


def test_honor_skyspell_patterns(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    config_path = tmp_path / "skyspell.kdl"
    config_path.write_text("patterns {\n  foo.lock\n}\n")
    open_file_with_contents(
        kak_checker, tmp_path / "foo.lock", r"I'm testing skyspell here"
    )

    kak_checker.send_command("skyspell-list")
    actual = kak_checker.get_option("skyspell_error_count")
    assert actual == "0"


def test_add_to_project(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker, tmp_path / "foo.txt", r"I'm testing skyspell here"
    )

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("p")
    kak_checker.send_command("quit")

    config = parse_config(tmp_path)
    project_words = [node.name for node in config['project'].nodes]
    assert project_words == ["skyspell"]


def test_add_to_file(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker, tmp_path / "foo.txt", r"I'm testing skyspell here"
    )

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("f")
    kak_checker.send_command("quit")

    config = parse_config(tmp_path)
    words = [node.name for node in config['paths']['foo.txt'].nodes]
    assert words == ["skyspell"]


def test_add_to_extension(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker, tmp_path / "foo.rs", "fn function(parameter: type) { body }"
    )

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("e")
    kak_checker.send_command("quit")

    config = parse_config(tmp_path)
    words = [node.name for node in config['extensions']['rs'].nodes]
    assert words == ["fn"]


def test_undo(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker, tmp_path / "foo.txt", r"I'm testing skyspell here"
    )

    kak_checker.send_command("skyspell-list")
    kak_checker.send_keys("a")
    kak_checker.send_keys("u")
    kak_checker.send_command("quit")

    config = parse_config(tmp_path)
    words = [node.name for node in config['global'].nodes]
    assert words == []


def test_replace_with_suggestion(tmp_path: Path, kak_checker: RemoteKakoune) -> None:
    open_file_with_contents(
        kak_checker, tmp_path / "foo.txt", "There is a missstake here"
    )

    kak_checker.send_command("skyspell-next")
    kak_checker.send_command("skyspell-replace")
    kak_checker.send_keys(r"\n")  # select first menu entry
    kak_checker.send_command("write-quit\n")

    actual = (tmp_path / "foo.txt").read_text()
    assert actual == "There is a mistake here\n"
