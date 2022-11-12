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

* Always skip `.skyspell-ignore`

# 0.3.0 (2022-03-12)

* Add `IgnoreFile`, `walker` (taken from `skyspell`)
* Handle Python string prefixes, like in `f'input`
* Remove `Interactor` and related code (moved to `skyspell`)


# 0.2.0 (2022-02-19)

* Don't store files to skip in the skyspell database

# 0.1.1 (2022-02-12)

* Remove aspell C wrapper - this was a fun experiment, but the additional
  complexity is not worth it (plus I don't like maintaining unsafe code)

Fix metadata

# 0.1.0 (2022-02-12)

Initial release

