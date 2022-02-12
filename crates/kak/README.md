# Kakoune integration for skyspell


## Installation

Install [skyspell](https://git.sr.ht/~dmerej/skyspell).

Install `skyspell_kak`:

```
$ cargo install skyspell_kak
```

Make sure that `skyspell-kak` is in your `PATH`.


Call `skyspell-kak init` in your `kakrc` file:

```
skyspell-kak init
```

## Usage

You can now call the various `:skyspell-` commands

Typical workflow:

* Call `skyspell-enable <LANG>` to install the `skyspell-check` hook.
* Edit files in the current project. All open buffers will be checked
  as soon as they're written.
* Use `skyspell-list` to list all error in a special `*spelling*` buffer
* For each line in `*spelling*`, execute the given action (see
  `skyspell-help` for details). You can for instance choose to add
  the error to the list of exceptions for the current project
* Once an error has been selected, you can also use `:skyspell-replace` to
  open a menu containing the replacements suggestions.

Note that instead of using `skyspell-list` you can also use `skyspell-next` and
`skyspell-previous`.

See also:

[![asciicast](https://asciinema.org/a/427100.svg)](https://asciinema.org/a/427100)
