# skyspell

A fast and handy spell checker for the command line.

# skyspell in action

Basically, you give `skyspell` a project path and a list of files to check,
then you choose how to handle the errors it finds:

```
$ skyspell check --project-path . $(git ls-files)
LICENSE:9:2 Redistributions
What to do?
...
> g

=> Added 'Redistributions' to the global ignore list

foo.rs:32:0 2fn
What to do?
...
> e

=> Added 'fn' to the ignore list for '.rs' files
```

## Features

* Distributed as a single binary
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
* Kakoune integration

## Comparison with scspell

I've borrowed heavily from [scspell](https://github.com/myint/scspell) -
both for the implementation and the command line behavior.

Note that scspell does not depend on Enchant and so can not check
Languages other than English, and also cannot offer suggestions for
spell errors.

But it's implementation is simpler and does not require to install a
spell provider.

On the other hard, scspell can apply replacements in a file automatically,
a feature `skyspell` does not have.
