"""
Some notes about how this works

- We have a prelude.kak that contains test helper

Then in scenarios/ with have one kak script per test

They all end with 'quit' - which means that if there's an exception,
kakoune will be still running - which is exactly what we want when a
test fails because it will make debugging the test really easy. Just make
sure to type 'quit! 1' to let pytest know the test failed.

In order to run the whole suite quicker, you can also set
SKYSPELL_NON_INTERACTIVE_TESTS to 1 and use `-n auto`.
"""

import os
import subprocess
from pathlib import Path
from typing import Any

import pytest
from pytest import MonkeyPatch


def get_scenarios() -> list[Path]:
    scenarios_path = Path("scenarios")
    return sorted(scenarios_path.glob("*.kak"))


def parse_config(path: Path) -> dict[str, Any]:
    contents = path.read_text()
    config: dict[str, Any] = tomllib.loads(contents)
    return config


def parse_local_ignore(tmp_path: Path) -> dict[str, Any]:
    return parse_config(tmp_path / "skyspell-ignore.toml")


def parse_global_ignore(tmp_path: Path) -> dict[str, Any]:
    return parse_config(tmp_path / "data" / "skyspell" / "global.toml")


@pytest.mark.parametrize("scenario", get_scenarios())
def test_scenario(scenario: Path, tmp_path: Path) -> None:

    print("\nRunning", scenario.name)
    kak_env = os.environ.copy()
    kak_env.pop("SKYSPELL_GLOBAL_PATH", None)
    kak_env["XDG_DATA_HOME"] = str(tmp_path / "data")
    this_path = Path(".")
    kak_env["SKYSPELL_TESTS_PATH"] = str(this_path.absolute())

    prelude = this_path / "prelude.kak"
    script = f"""
    source {prelude.absolute()}
    source {scenario.absolute()}
    """
    cmd = ["kak", "-n", "-e", script]
    try:
        if os.getenv("SKYSPELL_NON_INTERACTIVE_TESTS"):
            cmd.extend(["-ui", "dummy"])
            subprocess.run(
                cmd,
                cwd=tmp_path,
                check=True,
                env=kak_env,
                timeout=3,
            )

        else:
            subprocess.run(
                cmd,
                cwd=tmp_path,
                check=True,
                env=kak_env,
            )
    except subprocess.SubprocessError:
        pytest.fail(scenario.name)
