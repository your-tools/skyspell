# rcspell

A fast and handy spell checker for source code and other texts

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
