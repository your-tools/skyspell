# 5.0.0 (2025-11-17)

## Highlights

- Implement changes required for the brand new [VSCode Extension](https://github.com/your-tools/skyspell-vscode)
- Allow using the same language tags across UNIX and Windows.

## Bug fixes and improvements

- Fix using undo directly from the cli
- Make `--output-format json` imply `--non-interactive`
- Add --lang to the add/remove skyspell command line actions

## Breaking changes

- Move --output-format to the 'check' subcommand (it made no sense for the other sub commands)

- When using `--output-format=json`:
  - Show absolute instead of relative paths
  - Move unknown words into an `errors` key
  - Add a `suggestions` entry:

```json
// Old
{
  "file.md": [
    {
      "word": "missstake",
      "range": { "line": 3, "start_column": 1, "end_column": 9 }
    }
  ]
}
```

```json
// New
{
  "errors": {
    "/path/to/file.md": [
      {
        "word": "missstake",
        "range": { "line": 3, "start_column": 1, "end_column": 9 }
      }
    ]
  },
  "suggestions": {
    "missstake": ["miss stake", "miss-stake", "mistake", "misstate"]
  }
}
```

## Misc

- Rewrite kakoune integration tests using mostly kak scripts
- Bump many dependencies
- Bump to 2024 edition

# 4.0.0 (2025-01-06)

This release allows to skip entire tokens while processing text files.

See [skyspell_core changelog](https://github.com/your-tools/skyspell/blob/main/crates/core/CHANGELOG.md#new-feature-skipping-entire-tokens) for details.

Also, you must press 'g' instead of 'a' when using the interactive checker to add a word to the global
ignore list.

# 3.0.1 (2024-12-07)

Fix help message when using the interactive checker.

Also, bump `skyspell_core` to 4.0.1, which allows using the same config files from Unix and Windows.

# 3.0.0 (2024-12-07)

Bump `skyspell_core` to 4.0.0

This release adds support for Windows.

See [skyspell_core changelog](https://github.com/your-tools/skyspell/blob/main/crates/core/Changelog.md#400-2024-12-07) for details.

**Breaking**: The `--lang` option is now required. Note that for "English US", it should look like
`--lang=en` if using Enchant (on Unix), or just `--lang=en` on Windows.

# 2.0.1 (2024-10-20)

- Fix crate metadata
- Bump `dialoguer` dependency

# 2.0.0 (2024-10-15)

See [skyspell_core](https://github.com/your-tools/skyspell/blob/main/crates/core/Changelog.md#300-2024-10-15) changelog.

# 1.0.2 (2023-04-25)

- Bump `skyspell_core` to v2.0.0

# 1.0.1 (2022-11-12)

- Bump dependencies
- Update metadata

# 1.0.0 (2022-07-17)

## Changes in configuration files

See [skyspell_core](https://github.com/your-tools/skyspell/blob/main/crates/core/Changelog.md#100-2022-07-17) changelog.

## Changes in command-line syntax

The `--project-path` option must use right before the action.

Also, to add or ignore a rule for a project, use the boolean `--project` option

Before skyspell 1.0

```

# Add bar as a ignored word for the project in /path/to/foo

skyspell add --project-path /path/to/foo bar

# Add baz to the global ignore list:

skyspell add baz

```

After skyspell 1.0

```

# Add bar as a ignored word for the project in /path/to/foo

skyspell --project-path /path/to/bar add bar --project

# Add baz to the global ignore list:

skyspell --project-path add baz --project

```

In this version, the `skyspell-ignore.kdl` file in the current working
directory will be parsed, because we need to know if we need to store
baz in a sqlite or in the configuration file, which is why both
`--project-path` and `--project` need to be used.

# 0.8.1 (2022-03-12)

- Handle Python string prefixes, like in `f'input`

# 0.8.0 (2022-02-19)

## Breaking change: skip files using an ignore file

Remove "skip" features from the SQL repository and from the command line.

Instead of telling skyspell to skip `poetry.lock`, `Cargo.lock` and
`favicon.ico`, you can just create a file named `.skyspell-ignore` containing:

```

\*.lock
favicon.ico

```

This makes the code much faster because we don't need to make a sql query for each
file we check, just when we find a spelling error.

This also means you can run `skyspell-check` without specifying the files to check:

```

# old:

$ skyspell check --project-path . $(git ls-files)

# new:

$ skyspell check --project-path .

```

Or even without specifying `--project-path` at all, which defaults to the
current working directory.

# 0.7.1 (2022-02-12)

- Remove `--aspell` option and `aspell` support. Additional complexity to avoid
  going through Enchant does not seems worth it.
- Tweak skyspell cli output

# 0.7.0 (2022-02-12)

- **Breaking** Split code into separate crates. This means you now need to install `skyspell_kak` in
  order to use the Kakoune integration.
- **Breaking** Remove `skyspell_underline_errors` option. We now use a specific SellingError face
  and users can change the default value if needed.

## Internal

- Use `kak_quoted_buflist` instead of `kak_buflist`
- Bump to clap 3.0
- Switch from `chrono` to `time`

# 0.6.1 (2021-11-01)

- Bump to Rust 2021 edition
- Fix splitting text in tokens when an abbreviation is in the middle of
  the identifier
- Fix when using latest Kakoune
- Improve error message when trying to spell check binary files

# 0.6.0 (2021-10-15)

- **Breaking**: The `SKYSPELL_DB_PATH` environment variable is no longer taken into account
- **Breaking**: Change syntax for kakoune integration:

```diff
  evaluate-commands %sh{
- skyspell kak init
+ skyspell-kak init  # < note the '-' instead of the space
  }
```

- Better error message when files contain invalid UTF-8 data
- Add option `skyspell_underline_errors` : to use curly underline red
  for spelling errors. Requires kakoune > 2021.08.28 (after
  [this commit](https://github.com/mawww/kakoune/commit/3fc8e29d101b4f6eef2538cdbe799bab9859f4b3)

# 0.5.0 (2021-09-16)

- Implement "undo" for the interactive checker
- Improve performance
- Improve error handling
- Improve test coverage
- Properly parse "doesn't", "it's" and the like
- Fix bug regarding case sensitivity
- Be a bit more verbose when spell checking a project
- When trying to remove items from the repository, tell user if the item
  was already absent instead of silently doing nothing.

## Kakoune integration

- Add end-to-end testing using kitty's remote protocol and pytest
- Display errors in the status bar
- Display an error message when no suggestions are found
- Tell user when they are calling `suggest` and the selection is not an error
- Tell user when the selection is blank
- Add `undo` hook for the `*spelling*` buffer

# 0.4.0 (2021-06-15)

## Add support for projects

- File paths are now relative to the project root
- You can ignore words for a given project instead of globally

# 0.3.0 (2021-06-12)

- Allow to unskip paths and file names
- Better handling of `\` in source files

## Kakoune integration

- Use `a` to add to global ignore instead of `g` (it breaks `ga`)
- Check all open buffers, not just the current one
- Implement `next` and `previous`
- Use a `kak` subcommand instead of guessing whether we are called
  from Kakoune.

# 0.2.1 (2021-05-28)

Fix project metadata

# 0.2.0 (2021-05-28)

- Allow adding words to a global ignore list or by extension
- Add a non-interactive mode
- Allow to skip file names or file paths
- Check for good words with Enchant
- Support languages other than English
- Add support for suggestions
- Add Kakoune integration

# 0.1.0 (2021-05-18)

Initial release
