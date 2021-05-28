# rcspell

A fast and handy spell checker for source code and other texts

# rcspell in action

rcspell is meant to be used from the command line, like this:

```
$ rcspell check $(git ls-files)
/path/to/LICENSE:9:2 Redistributions
What to do?
...
> g

=> Added Redistributions to the global ignore list

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
* Ignore list per file extension (like `fn` for `.rs`)
* Ignore list per file path
* Skip list per file name (like always skipping file named `Cargo.lock`)
* Skip list per full path
* Kakoune integration
