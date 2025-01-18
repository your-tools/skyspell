"""

Helper script to parse toml files from kak script

See prelude.kak for the details

"""

import sys
import tomllib
from pathlib import Path
from typing import Any


def parse_config(path: Path) -> dict[str, Any]:
    contents = path.read_text()
    config: dict[str, Any] = tomllib.loads(contents)
    return config


def parse_local_ignore() -> dict[str, Any]:
    return parse_config(Path("skyspell-ignore.toml"))


def parse_global_ignore() -> dict[str, Any]:
    return parse_config(Path("data") / "skyspell" / "global.toml")


def main() -> None:
    _, type, *keys, expected_value = sys.argv
    key_path = ".".join(keys)
    match type:
        case "local":
            config = parse_local_ignore()
        case "global":
            config = parse_global_ignore()
        case other:
            sys.exit(f"Unknown config type: {type}")
    value = config
    try:
        for key in keys:
            value = value[key]
    except KeyError:
        print(f'fail %[In {type} config file: key "{key_path}" not found]')

    if expected_value not in value:
        print(
            f'fail %[In {type} config file: value "{expected_value}" not fond for "{key_path}"]'
        )


if __name__ == "__main__":
    main()
