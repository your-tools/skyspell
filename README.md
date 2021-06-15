# skyspell

A fast and handy spell checker for the command line.

# skyspell in action

skyspell is meant to be used from kakoune, but also features
an interactive mode and can be used from the command line, like this:

```
$ skyspell check $(git ls-files)
/path/to/LICENSE:9:2 Redistributions
What to do?
...
> g

=> Added 'Redistributions' to the global ignore list

/path/to/foo.rs:32:0 2fn
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
   * it can parse `camelCase` identifiers
   * it knows how to skip URLs, sha1s and the like
* Global ignore list
* Ignore list per file extension (like `fn` for `.rs`), projects, or
  relative path inside projects
* Skip list per file name (like always skipping file named `Cargo.lock`)
* Skip list per relative path inside a project (like `image.svg`)
* Kakoune integration

## Inspiration

I've borrowed heavily from [scspell](https://github.com/myint/scspell) -
both for the implementation and the command line behavior.

Not that scspell does not depend on Enchant and so can not check Languages other than English, and also
cannot offer suggestions for spell errors.

But it's implementation is simpler and does not require to install a
spell provider.

On the other hard, scspell can replace suggestions in the file.

So use the one which suits you best, I don't really care :)

