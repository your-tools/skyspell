# 3.0.1 - Unreleased

Revert the change from 'a' to 'g' when using the `*spelling*` buffer to add a word to the global ignore list.

It conflicts with other useful 'g' commands like `ga`.

# 3.0.0 (2025-01-06)

This release allows to skip entire tokens while processing text files.

See [skyspell_core changelog](https://github.com/your-tools/skyspell/blob/main/crates/core/CHANGELOG.md#new-feature-skipping-entire-tokens) for details.

Also, you must press 'g' instead of 'a' when using the `*spelling*` buffer to add a word to the global
ignore list.

# 2.0.1 (2024-10-20)

Fix crate metadata

# 2.0.0 (2024-10-15)

Bump `skyspell_core` to 3.0.0

See [skyspell_core](https://github.com/your-tools/skyspell/blob/main/crates/core/Changelog.md#300-2024-10-15) changelog.

# 1.0.3 (2022-04-25)

Bump `skyspell_core` to 2.0.0

# 1.0.2 (2022-04-25)

Bug fix: use a draft context when filling up the `*spelling*` buffer.
Original patch by @micahh2


# 1.0.1 (2022-11-12)

Bug fix: using `.` after `skyspell-check` now works as expected.

# 1.0.0 (2022-07-17)

## Changes in configuration files

See [skyspell_core](https://github.com/your-tools/skyspell/blob/main/crates/core/Changelog.md) changelog.

## Changes in behavior

When the `skyspell-ignore.kdl` file is used, the `undo` operation is now a non-op.

This should be fine because you can always open the `skyspell-
ignore.kdl` buffer and undo the changes there. Even better, the `.kdl`
file can be tracked by whatever version control system you're using for
your project.

# 0.8.2 (2022-04-16)

* Rename option `spell_errors` -> `skyspell_errors`
* Reset skyspell options when using `skyspell-disable`

# 0.8.1 (2022-03-12)

* Handle Python string prefixes, like in `f'input`

# 0.8.0 (2022-02-19)

## Breaking change: skip files using an ignore file

Remove "skip" features from the SQL repository and from the `*spelling*` buffer.

Instead of telling skyspell to skip `poetry.lock`, `Cargo.lock` and
`favicon.ico`, you can just create a file named `.skyspell-ignore` containing:

```
*.lock
favicon.ico
```

This makes the code much faster because we don't need to make a sql query for each
file we check, just when we find a spelling error.

Note that you have to edit the file "by hand" now


.8.0
# 0.7.0 (2022-02-12)

Initial release


