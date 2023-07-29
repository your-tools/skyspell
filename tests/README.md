# Integration tests for skyspell

## Kakoune integration tests

Testing kakoune integration is a bit hard using just Rust code, so instead we use end-to-end testing
via the remote control protocol of kitty.

Why kitty?
 * Because it's fast
 * Because the escape key works reliably there
 * Because the remote protocol is simple to use

You can think of theses tests as tests for a web application using Selenium.

They are a bit fragile because they rely on what is actually *displayed on screen*,
but on the other hand they are fun to write and tests are easy to debug.

*Note*: due to the nature of end-to-end testing, the window in which kakoune tests
are running must be big enough for the text to be visible. Using i3, this can be achieved
with:

```
for_window [class="kitty-tests"] floating enable , resize set 800 700
```

This works because the tests set the window class when they start the remote kakoune instances.

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
