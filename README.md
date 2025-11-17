# skyspell

A fast and handy spell checker for the command line.

## Features

* Handy command line interface
* Leverages the excellent [enchant](https://abiword.github.io/enchant/) library,
  so compatible with existing providers and dictionaries
* Hand-made tokenizer, which means
   * it can parse `camelCase` , `snake_case` identifiers
   * it knows how to skip URLs, sha1s and the like
   * it handles abbreviations like in `HTTPError`
   * ... and more!
* Global ignore list
* Ignore list per file extension (like `fn` for `.rs`), projects, or
  relative path inside projects
* Skip list per file names (like always skipping files named `Cargo.lock`)
* Skip list per relative path inside a project (like `image.svg`)
* All of the above are stored in a toml files, which makes it easy to backup/restore
  your ignore rules, or use them for CI.
* Editor integrations:
  * [Kakoune](https://github.com/your-tools/skyspell/blob/main/crates/kak/README.md)
  * [VSCode](https://github.com/your-tools/skyspell-vscode/)

## Installation

### On Linux 

You will need:

* The C Enchant library installed, with its development headers
* One of Enchant's backends (aspell, hunspell, nuspell)
* A dictionary for the language you'll be using matching one of
  the above backends (like `aspell-en` or `hunspell-fr`).
* `cargo`

Then run:

```
$ cargo install skyspell
```

### On Windows

Use the above method, or see the installation instructions in the "Releases" section on GitHub.

### On macOS

See #9 :P

## Checking setup

Run `skyspell suggest helllo`, and check that the word `hello`
is suggested.

## skyspell in action

Usually, you will run `skyspell check` to start an interactive session,
where you tell `skyspell` how to handle all the errors it finds in your
project files:

```
$ skyspell check
LICENSE:9:2 Redistributions
What to do?
g : Add word to global ignore list
e : Add word to ignore list for this extension
...
x : Skip this error
q : Quit
> : a
=> Added 'Redistributions' to the global ignore list

foo.rs:32:2 fn
What to do?
g : Add word to global ignore list
e : Add word to ignore list for this extension
...
q : Quit
x : Skip this error
> : e
=> Added 'fn' to the ignore list for '.rs' files
```

Ignore rules will be automatically added to either:

- `skyspell-ignore.toml`, the local file, if the word is ignored for the project or for a path
- or in `~/.local/share/skyspell/global.toml`, the global file, if the word is ignored globally
  or for a given extension.

That way you can share your ignore rules with other users, or back them up anyway you like.

Note that skyspell will honor `XDG_DATA_DIR` when looking for the global file.

## Excluding files from the check

Note that by default, skyspell will try to read *every* file in the
project, or if the project is using git, every file not ignored by git.
This may include for instance binary files.

To prevent skyspell from trying to read those files, create a
`skyspell-ignore.toml` file  at the root of your project containing
something like this:

```toml
patterns = [
   "Cargo.lock",
   "logo.png ",
]
```


## Comparison with scspell

I've borrowed heavily from [scspell](https://github.com/myint/scspell) -
both for the implementation and the command line behavior.

Note that scspell does not depend on Enchant and so can not check
Languages other than English, and also cannot offer suggestions for
spell errors.

But it's implementation is simpler and does not require to install a
spell provider.

On the other hand, scspell can apply replacements in a file automatically,
a feature `skyspell` does not have.

