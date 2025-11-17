# 5.0.0 (2025-01-06)

## Breaking: new `TokenProcessor` API

`TokenProcessor::new()` now takes a file name and a `BufReader`,
instead of just `Path`

Implement `Iterator` for TokenProcessor. TokenProcessor::Item is a
Result<Token>, where `Token` is a new struct containing the word and its
position (line and column number)

Get rid of `TokenProcessor::each_token`

This makes the API more idiomatic:

```rust
// Old API, for skyspell_core <= 4.0.0
let relative_path = RelativePath::new(project_path, source_path)?;
let token_processor = TokenProcessor::new(source_path);
token_processor.each_token(|token, line, column| {
    // Do something with token, line, column
 })?;

// New API: for skyspell_core >= 5.0.0
let reader = BufReader::new(...);
let file_name = ...;
let mut token_processor = TokenProcessor::new(reader, &file_name);
for token in token_processor {
    let token = token?;
    // Do something with token.text, token.pos
}
```

This also seems to improve performance a bit.

## New feature: skipping entire tokens

The first thing skyspell does while cheeking a text file is to split the contents into "tokens".
Then it tries to split tokens into individual words to be checked, for instance the token
`TokenProcessor` gives the words `Token` and `Processor`.

But sometimes you need to skip a token completely, like if you have a base64 string:

```js
// in tests.js
base64value = "DeadBeef==";
```

In this case, you can skip the token in `skyspell-ignore.toml`, like this:

```toml
[skipped]
"tests.js" = [
  "DeadBeef==",
]
```

# 4.0.1 (2024-12-07)

- Normalize 'lang' when reading/writing in the `global.toml` configuration file
- Normalize relative paths when reading/writing in the `skyspell-ignore.toml` configuration file
- Allow to specify `global.toml` path the `SKYSPELL_GLOBAL_PATH` environment variable

# 4.0.0 (2024-12-07)

- Add support for Windows. The spell checking is done by using the Win32 APIs, instead of
  Enchant (which is hard to build and distribute on this platform)

# 3.0.1 (2024-10-20)

- Fix crate metadata

# 3.0.0 (2024-10-15)

**Breaking** : remove SQL storage - all the ignore rules are now stored in plain `toml` files.

New feature: allow to ignore words based on the current lang

# 2.0.0 (2022-11-12)

**Breaking** : most methods on the public traits are now `mut`, even the
ones which do not modify the database - this was due to the diesel v2
upgrade.

Also, bump clap from v3 to v4

# 1.0.3 (2022-11-12)

Bump dependencies

# 1.0.2 (2022-07-19)

## Fixes

Display more info when parsing `skyspell-ignore.kdl` fails. In particular
the filename, line number and column number are displayed, along with
an help message if it exists.

# 1.0.1 (2022-07-18)

## Fixes

There was a bug in the code that writes the generated `kdl` file. In some cases, the text was generated this way:

```
{
  extensions rs {
    struct
  }}
}
```

which is invalid and caused subsequent calls to skyspell to fail.

This has been fixed.

# 1.0.0 (2022-07-17)

## Change in configuration file

Instead of a `.skyspell-ignore` file using `gitignore` syntax,
configuration is now read from a [kdl](https://kdl.dev/) file named
`skyspell-ignore.kdl`

Before skyspell 1.0:

```
# in .skyspell-ignore
*.lock
```

After skyspell 1.0

```
# in skyspell-ignore.kdl
patterns {
   *.lock
}
```

## New storage backend

In addition to storing ignore rules in an `sqlite` database, you can
now store ignore rule in a `skyspell-ignore.kdl` file.

See the [skyspell-ignore.kdl for this project](https://github.com/your-tools/skyspell/blob/main/skyspell-ignore.kdl)
for an example.

By default, the storage backend will be this configuration file. This
makes it easier to share ignore rules across member of the same project,
or even to run `skyspell` as part of a continuous integration system.

The old storage backend can still be used by inserting the correct node
in the file, like this:

```kdl
use_db

# still used to skip unwanted files
patterns {
    *.lock
}

# no longer used
project {
}

extensions {
}

# ...
```

# 0.3.1 (2022-03-12)

- Always skip `.skyspell-ignore`

# 0.3.0 (2022-03-12)

- Add `IgnoreFile`, `walker` (taken from `skyspell`)
- Handle Python string prefixes, like in `f'input`
- Remove `Interactor` and related code (moved to `skyspell`)

# 0.2.0 (2022-02-19)

- Don't store files to skip in the skyspell database

# 0.1.1 (2022-02-12)

- Remove aspell C wrapper - this was a fun experiment, but the additional
  complexity is not worth it (plus I don't like maintaining unsafe code)

Fix metadata

# 0.1.0 (2022-02-12)

Initial release
