# 0.7.0

* **Breaking** Split code into separate crates. This means you now need to install `skyspell_kak` in
  order to use the Kakoune integration.
* **Breaking** Remove `skyspell_underline_errors` option. We now use a specific SellingError face
  and users can change the default value if needed.

# 0.6.1 (2021-11-01)

* Bump to Rust 2021 edition
* Fix splitting text in tokens when an abbreviation is in the middle of
  the identifier
* Fix when using latest Kakoune
* Improve error message when trying to spell check binary files

# 0.6.0 (2021-10-15)

* **Breaking**: The `SKYSPELL_DB_PATH` environment variable is no longer taken into account
* **Breaking**: Change syntax for kakoune integration:

```diff
  evaluate-commands %sh{
- skyspell kak init
+ skyspell-kak init  # < note the '-' instead of the space
  }
```

* Better error message when files contain invalid UTF-8 data
* Add option `skyspell_underline_errors` : to use curly underline red
  for spelling errors. Requires kakoune > 2021.08.28 (after
  [this commit](https://github.com/mawww/kakoune/commit/3fc8e29d101b4f6eef2538cdbe799bab9859f4b3)

# 0.5.0 (2021-09-16)

* Implement "undo" for the interactive checker
* Improve performance
* Improve error handling
* Improve test coverage
* Properly parse "doesn't", "it's" and the like
* Fix bug regarding case sensitivity
* Be a bit more verbose when spell checking a project
* When trying to remove items from the repository, tell user if the item
  was already absent instead of silently doing nothing.

## Kakoune integration

* Add end-to-end testing using kitty's remote protocol and pytest
* Display errors in the status bar
* Display an error message when no suggestions are found
* Tell user when they are calling `suggest` and the selection is not an error
* Tell user when the selection is blank
* Add `undo` hook for the `*spelling*` buffer

# 0.4.0 (2021-06-15)

## Add support for projects

* File paths are now relative to the project root
* You can ignore words for a given project instead of globally

# 0.3.0 (2021-06-12)

* Allow to unskip paths and file names
* Better handling of `\` in source files

## Kakoune integration

* Use `a` to add to global ignore instead of `g` (it breaks `ga`)
* Check all open buffers, not just the current one
* Implement `next` and `previous`
* Use a `kak` subcommand instead of guessing whether we are called
  from Kakoune.

# 0.2.1 (2021-05-28)

Fix project metadata

# 0.2.0 (2021-05-28)

* Allow adding words to a global ignore list or by extension
* Add a non-interactive mode
* Allow to skip file names or file paths
* Check for good words with Enchant
* Support languages other than English
* Add support for suggestions
* Add Kakoune integration

# 0.1.0 (2021-05-18)

Initial release


