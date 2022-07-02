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
* [Kakoune integration](https://git.sr.ht/~your-tools/skyspell/tree/main/item/crates/kak/README.md)

## Installation

You will need:

* The C Enchant library installed, with its development headers
* The sqlite3 library installed, which its development headers
* One of Enchant's backends (aspell, hunspell, nuspell)
* A dictionary for the language you'll be using matching one of
  the above backends (like `aspell-en` or `hunspell-fr`).
* `cargo`

Then run:

```
$ cargo install skyspell
```

and make sure `skyspell` is in your `PATH`.

## skyspell in action

Usually, you will run `skyspell check` to start an interactive session,
where you tell `skyspell` how to handle all the errors it finds in your
project files:

```
$ skyspell check
LICENSE:9:2 Redistributions
What to do?
a : Add word to global ignore list
e : Add word to ignore list for this extension
p : Add word to ignore list for the current project
f : Add word to ignore list for the current file
x : Skip this error
q : Quit
> : g
=> Added 'Redistributions' to the global ignore list

foo.rs:32:2 fn
What to do?
a : Add word to global ignore list
e : Add word to ignore list for this extension
p : Add word to ignore list for the current project
f : Add word to ignore list for the current file
x : Skip this error
q : Quit
> : e
=> Added 'fn' to the ignore list for '.rs' files
```

## Advanced usage

If for some reason a file can't be checked, you can create a `.skyspell-ignore` file,
like this:

```
Cargo.lock
```

See also `skyspell --help` for the various command and flags.

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

## Local development

To build faster and run the tests faster, you can use

* [mold](https://github.com/rui314/mold/)
* [cargo nextest](https://nexte.st/)
