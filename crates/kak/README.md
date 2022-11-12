# Kakoune integration for skyspell


## Installation

Install [skyspell](https://github.com/your-tools/skyspell).

Install `skyspell_kak`:

```
$ cargo install skyspell_kak
```

Make sure that `skyspell-kak` is in your `PATH`.


Call `skyspell-kak init` in your `kakrc` file:

```
evaluate-commands %sh{
  skyspell-kak init
}
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

Note that instead of using `skyspell-list` and then `Enter` to jump from
on spelling error to the next, you can also use `skyspell-next` and
`skyspell-previous`.

## Customization

It's advised to create a `skyspell` user mode:

For instance:

```
map global user s ': enter-user-mode skyspell<ret>' -docstring 'enter spell user mode'
map global skyspell d ': skyspell-disable<ret>' -docstring 'clear spelling highlighters'
map global skyspell e ': skyspell-enable en_US<ret>' -docstring 'enable spell checking in English'
map global skyspell l ': skyspell-list <ret>' -docstring 'list spelling errors in a buffer'
map global skyspell h ': skyspell-help <ret>' -docstring 'show help message'
map global skyspell n ': skyspell-next<ret>' -docstring 'go to next spell error'
map global skyspell p ': skyspell-previous<ret>' -docstring 'go to previous spell error'
map global skyspell r ': skyspell-replace<ret>' -docstring 'suggest a list of replacements'
```

skyspell also declares a face named `SpellingError` that you can change if you want
to show spelling errors in a different way.
