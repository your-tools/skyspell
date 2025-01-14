import os
import subprocess
import time
import tomllib
from pathlib import Path
from typing import Any, Iterator

import pytest


class TmuxSession:
    def __init__(self, *, tmp_path: Path) -> None:
        self.socket_path = tmp_path / "tmux.sock"
        env = os.environ.copy()
        env["PS1"] = r"\w$ "
        env["PATH"] = os.environ["PATH"]
        env["XDG_DATA_HOME"] = str(tmp_path / "data")
        print(f"tmux listening on {self.socket_path}")
        self.session = "session"
        self.pane = "skyspell-tests"

        # fmt: off
        subprocess.run(
            [
                "tmux", "-S", self.socket_path, "new-session",
                "-s", self.session, "-n", self.pane, "-d", "sh",
            ],
            env=env,
            check=True,
        )
        # fmt: on

    def send_keys(self, *keys: str) -> None:
        # fmt: off
        subprocess.run(
            [
                "tmux", "-S", self.socket_path, "send-keys",
                "-t", f"{self.session}:{self.pane}",
                *keys,
            ]
        )
        # fmt: on

    def get_text(self) -> str:
        # fmt: off
        process = subprocess.run(
            [
                "tmux", "-S", self.socket_path, "capture-pane",
                "-t", f"{self.session}:{self.pane}", "-p",
            ],
            check=True,
            capture_output=True,
            text=True,
        )
        # fmt: on
        return process.stdout

    def terminate(self) -> None:
        subprocess.run(["tmux", "-S", self.socket_path, "kill-server"])


@pytest.fixture()
def tmux_session(tmp_path: Path) -> Iterator[TmuxSession]:
    session = TmuxSession(tmp_path=tmp_path)
    yield session
    session.terminate()


class RemoteKakoune:
    def __init__(self, tmux_session: TmuxSession) -> None:
        self.tmux_session = tmux_session
        self.tmux_session.send_keys("kak -n", "Enter")

    def send_command(self, command: str, *args: str) -> None:
        self.tmux_session.send_keys("Escape")
        text = f":{command} " + " ".join(args)
        print(text)
        self.send_keys(text)
        self.send_keys("Enter")

    def send_keys(self, *args: str) -> None:
        self.tmux_session.send_keys(*args)

    def get_option(self, option: str) -> str:
        prefix = f"option : {option} => "
        self.send_command("echo", "-debug", prefix, f"%opt[{option}]")
        extracted = self.extract_from_debug_buffer(prefix)
        if not extracted:
            pytest.fail(f"Kakoune option '{option}' not found")
        return extracted

    def get_selection(self) -> str:
        prefix = "selection => "
        self.send_command("echo", "-debug", prefix, "%val{selection}")
        extracted = self.extract_from_debug_buffer(prefix)
        if not extracted:
            pytest.fail(f"No value fond for kakoune selection")
        return extracted

    def extract_from_debug_buffer(self, prefix: str) -> str  | None:
        time.sleep(0.5)
        self.send_command("edit", "-existing", "*debug*")
        text = self.tmux_session.get_text()
        matching_lines = [x for x in text.splitlines() if x.startswith(prefix)]
        if not matching_lines:
            return None
        # If the value has changed, we want the latest
        line = matching_lines[-1]
        res = line[len(prefix) :]
        self.send_command("buffer-previous")
        return res


def parse_config(path: Path) -> dict[str, Any]:
    # Note: we need to sleep because there's a delay between
    #   - the key has been sent to the tmux window
    #   - it has been processed by kakoune
    #   - and the file has been written
    time.sleep(0.5)
    contents = path.read_text()
    config: dict[str, Any] = tomllib.loads(contents)
    return config


def parse_local_ignore(tmp_path: Path) -> dict[str, Any]:
    return parse_config(tmp_path / "skyspell-ignore.toml")


def write_local_ignore(tmp_path: Path, contents: str) -> None:
    (tmp_path / "skyspell-ignore.toml").write_text(contents)


def parse_global_ignore(tmp_path: Path) -> dict[str, Any]:
    return parse_config(tmp_path / "data" / "skyspell" / "global.toml")


class KakChecker:
    """
    Represent an instance of kakoune running in a tmux session

    Note that all runtime errors are caught and written to a file named 'err.txt'
    """

    def __init__(self, kakoune: RemoteKakoune, tmp_path: Path) -> None:
        self.kakoune = kakoune
        self.tmp_path = tmp_path
        hook_command = "%{echo -to-file @runtime_errors_path@ %val{hook_param}}"
        hook_command = hook_command.replace(
            "@runtime_errors_path@", str(self.runtime_errors_path)
        )
        self.kakoune.send_command(
            "hook",
            "global",
            "RuntimeError",
            ".+",
            hook_command,
        )
        self.kakoune.send_command("source", '"%val{runtime}/autoload/tools/menu.kak"')
        self.kakoune.send_command("evaluate-commands", "%sh{ skyspell-kak init }")
        self.kakoune.send_command("skyspell-enable", "en")
        self._errors_expected = False

    @property
    def runtime_errors_path(self) -> Path:
        return self.tmp_path / "runtime_errors.txt"

    def debug(self) -> None:
        subprocess.run(
            [
                "kitty",
                "--detach",
                "tmux",
                "-S",
                self.kakoune.tmux_session.socket_path,
                "attach",
            ]
        )
        breakpoint()

    def open_file_with_contents(self, path: str, text: str) -> None:
        full_path = self.tmp_path / path
        full_path.write_text(text)
        self.kakoune.send_command(f"edit %{{{path}}}")
        self.kakoune.send_command("skyspell-check")

    def open_error_list(self) -> None:
        self.kakoune.send_command("skyspell-list")

    def jump_next(self) -> None:
        self.send_command("skyspell-next")

    def jump_previous(self) -> None:
        self.send_command("skyspell-previous")

    def get_selection(self) -> str:
        return self.kakoune.get_selection()

    def send_command(self, command: str, *args: str) -> None:
        self.kakoune.send_command(command, *args)

    def send_keys(self, *args: str) -> None:
        self.kakoune.send_keys(*args)

    def ignored(self) -> list[str]:
        config = parse_global_ignore(self.tmp_path)
        res: list[str] = config["global"]
        return res

    def ignored_for_project(self) -> list[str]:
        config = parse_local_ignore(self.tmp_path)
        res: list[str] = config["project"]
        return res

    def ignored_for_path(self, path: str) -> list[str]:
        config = parse_local_ignore(self.tmp_path)
        res: list[str] = config["paths"][path]
        return res

    def ignored_for_extension(self, extension: str) -> list[str]:
        config = parse_global_ignore(self.tmp_path)
        res: list[str] = config["extensions"][extension]
        return res

    def ignored_for_lang(self, lang: str) -> list[str]:
        config = parse_global_ignore(self.tmp_path)
        res: list[str] = config["lang"][lang]
        return res

    def move_cursor(self, line: int, column: int) -> None:
        self.kakoune.send_keys(f"{line}g")
        self.kakoune.send_keys(f"{column}l")

    def expect_runtime_error(self, expected_message: str) -> None:
        self._errors_expected = True
        all_errors = self.runtime_errors_path.read_text()
        assert expected_message in all_errors

    def check_runtime_errors(self) -> None:
        if self._errors_expected:
            return

        if self.runtime_errors_path.exists():
            pytest.fail(
                f"Some kakoune errors occurred during the test:\n{self.runtime_errors_path.read_text()}"
            )

    @property
    def error_count(self) -> int:
        value = self.kakoune.get_option("skyspell_error_count")
        return int(value)

    def quit(self) -> None:
        self.kakoune.send_command("quit!")


@pytest.fixture
def kak_checker(tmp_path: Path, tmux_session: TmuxSession) -> Iterator[KakChecker]:
    tmux_session.send_keys(f"cd {tmp_path}", "Enter")
    remote_kakoune = RemoteKakoune(tmux_session)
    kak_checker = KakChecker(remote_kakoune, tmp_path)
    yield kak_checker

    kak_checker.check_runtime_errors()
    kak_checker.quit()


def test_skip_patterns(tmp_path: Path, kak_checker: KakChecker) -> None:
    write_local_ignore(
        tmp_path,
        """
        patterns = ["*.lock"]
        """,
    )
    kak_checker.open_file_with_contents("foo.lock", r"I'm testing skyspell here")

    kak_checker.open_error_list()

    assert kak_checker.error_count == 0


def test_no_spelling_errors(kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents("foo.txt", "There is no mistake there\n")

    assert kak_checker.error_count == 0


def test_jump_to_first_error(kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents(
        "foo.txt",
        "There is a misstake here\nand an othhher one there",
    )
    assert kak_checker.error_count == 2
    kak_checker.open_error_list()

    kak_checker.send_keys("Enter")
    assert kak_checker.get_selection() == "misstake"


def test_do_not_break_dot(tmp_path: Path, kak_checker: KakChecker) -> None:
    original_contents = "There is a misstake here\n"
    foo_path = tmp_path / "foo.txt"
    kak_checker.open_file_with_contents(
        "foo.txt",
        original_contents,
    )
    kak_checker.open_error_list()
    kak_checker.send_command(f"edit {foo_path}")
    kak_checker.kakoune.send_keys(".")
    kak_checker.kakoune.send_command("write")
    new_contents = foo_path.read_text()
    assert original_contents == new_contents


def test_goto_next(kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents(
        "foo.txt",
        "There is a misstake here\nand an othhher one there",
    )
    # Make sure we are between the first and the second error
    kak_checker.move_cursor(1, 22)
    kak_checker.jump_next()
    assert kak_checker.get_selection() == "othhher"


def test_goto_previous(kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents(
        "foo.txt",
        "There is a misstake here\nand an othhher one there",
    )
    # Make sure we are between the first and the second error
    kak_checker.move_cursor(1, 22)
    kak_checker.jump_previous()
    assert kak_checker.get_selection() == "misstake"


def test_add_global(kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents("foo.txt", "I'm testing skyspell here")
    kak_checker.open_error_list()
    kak_checker.send_keys("g")

    assert kak_checker.ignored() == ["skyspell"]


def test_add_to_project(kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents("foo.txt", r"I'm testing skyspell here")
    kak_checker.open_error_list()
    kak_checker.send_keys("p")

    assert kak_checker.ignored_for_project() == ["skyspell"]


def test_add_to_file(kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents("foo.txt", r"I'm testing skyspell here")
    kak_checker.open_error_list()
    kak_checker.send_keys("f")

    assert kak_checker.ignored_for_path("foo.txt") == ["skyspell"]


def test_add_to_extension(kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents("foo.rs", r"I'm testing skyspell here")

    kak_checker.open_error_list()
    kak_checker.send_keys("e")

    assert kak_checker.ignored_for_extension("rs") == ["skyspell"]


def test_add_to_lang(kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents("foo.rs", r"I'm testing skyspell here")

    kak_checker.open_error_list()
    kak_checker.send_keys("l")

    assert kak_checker.ignored_for_lang("en") == ["skyspell"]


def test_undo(tmp_path: Path, kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents("foo.txt", "I'm testing skyspell here")
    kak_checker.open_error_list()
    kak_checker.send_keys("g")
    kak_checker.send_keys("u")

    assert kak_checker.ignored() == []


def test_replace_with_suggestion(tmp_path: Path, kak_checker: KakChecker) -> None:
    kak_checker.open_file_with_contents("foo.txt", "There is a mistake here")

    kak_checker.jump_next()

    kak_checker.send_command("skyspell-replace")
    time.sleep(0.5)
    kak_checker.send_keys("Enter")  # select first menu entry
    time.sleep(0.5)
    kak_checker.send_command("write-quit")

    time.sleep(0.5)
    actual = (tmp_path / "foo.txt").read_text()
    assert actual == "There is a mistake here\n"


def test_on_skyspell_failure(tmp_path: Path, kak_checker: KakChecker) -> None:
    write_local_ignore(
        tmp_path,
        """
        syntax_error = [
        """,
    )

    kak_checker.open_file_with_contents("foo.txt", "There is a misstake here")
    time.sleep(0.5)

    kak_checker.expect_runtime_error("skyspell-kak failed")
