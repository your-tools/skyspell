# Integration tests for skyspell

## Kakoune integration tests

Testing kakoune integration is a bit hard using just Rust code, so instead we use end-to-end testing
using the tmux command line.

They are a bit fragile because they rely on what is actually *displayed on screen*,
but on the other hand they are fun to write and easy to debug - and they sometimes can catch regressions 😎

## How to run the tests

Make sure to:

* Install `kakoune`
* Install `skyspell-kak` from this repo and not crates.io

```
$ cargo install --locked --path ../crates/kak/
# Re-run this each time the Rust code changes, including
# ../crates/kak/src/init.kak
```

* Have `~/.cargo/bin/` first in your PATH

Then run the tests with :

```
$ poetry install
$ poetry run pytest
```
