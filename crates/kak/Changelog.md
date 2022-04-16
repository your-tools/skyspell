# 0.8.2 (2022-04-16)

* Rename option `spell_errors` -> `skyspell_errors`
* Reset skyspell options when using `skyspell-disable`

# 0.8.1 (2022-03-12)

* Handle Python string prefixes, like in `f'input`

# 0.8.0 (2022-02-19)

## Breaking change: skip files using an ignore file

Remove "skip" features from the SQL repository and from the `*spelling*` buffer.

Instead of telling skyspell to skip `poetry.lock`, `Cargo.lock` and
`favicon.ico`, you can just create a file named `.skyspell-ignore` containing:

```
*.lock
favicon.ico
```

This makes the code much faster because we don't need to make a sql query for each
file we check, just when we find a spelling error.

Note that you have to edit the file "by hand" now


.8.0
# 0.7.0 (2022-02-12)

Initial release


