<a href="https://mod.io"><img src="https://static.mod.io/v1/images/branding/modio-color-dark.svg" alt="mod.io" width="400"/></a>

# modiom
[![CI][gha-badge]][gha-url]

[gha-badge]: https://github.com/nickelc/modiom/workflows/CI/badge.svg
[gha-url]: https://github.com/nickelc/modiom/actions?query=workflow%3ACI

modiom is a command line tool for [mod.io](https://mod.io) to search, download and update mods for games without builtin support.

1. [Building](#building)
2. [Installation](#installation)
3. [Usage](#usage)
    1. [`modiom login`](#modiom-login)
    2. [`modiom search`](#modiom-search)
    3. [`modiom info`](#modiom-info)
    3. [`modiom subs`](#modiom-subscriptions)
        1. [`modiom subs list`](#modiom-subscriptions-list)
        2. [`modiom subs add`](#modiom-subscriptions-add)
        3. [`modiom subs rm`](#modiom-subscriptions-remove)
    4. [`modiom download`](#modiom-download)
    5. [`modiom upload`](#modiom-upload)
4. [Manifest format](#the-modio-manifest-format)

## Building

modiom is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.
Building is easy:

```
$ git clone https://github.com/nickelc/modiom.git
$ cd modiom
$ cargo build --release
$ ./target/release/modiom --version
modiom 0.1.0
```

## Installation

### Cargo

```
$ git clone https://github.com/nickelc/modiom.git
$ cargo install --path modiom
```

## Usage

### modiom login

```
$ modiom login --help
modiom-login

USAGE:
    modiom login [OPTIONS] [api-key] [token]

OPTIONS:
        --test-env    use the test environment
    -h, --help        Prints help information

ARGS:
    <api-key>
    <token>
```

### modiom search

```
$ modiom search --help
modiom-search

USAGE:
    modiom search [OPTIONS] [--] [FULLTEXT]

OPTIONS:
        --game-id <ID>
        --id <ID>...
        --name <VALUE>
        --name-id <VALUE>
        --expr <EXPR>...
        --test-env           use the test environment
    -h, --help               Prints help information

ARGS:
    <FULLTEXT>
```

### modiom info

```
$ modiom info --help
modiom-info
Show information of mods

USAGE:
    modiom info [OPTIONS] <GAME> <MOD>

OPTIONS:
        --files       List all files.
        --stats       Show the statistics.
        --test-env    Use the mod.io test environment
    -h, --help        Prints help information

ARGS:
    <GAME>    Unique id of a game.
    <MOD>     Unique id of a mod.
```

### modiom subscriptions

```
$ modiom subs --help
modiom-subscriptions
Show information of subscriptions

USAGE:
    modiom subscriptions [OPTIONS] <SUBCOMMAND>

OPTIONS:
        --test-env    Use the mod.io test environment
    -h, --help        Prints help information

SUBCOMMANDS:
    list
    add
    remove
```

### modiom subscriptions list

```
$ modiom subs list --help
modiom-subscriptions-list

USAGE:
    modiom subscriptions list [OPTIONS]

OPTIONS:
        --game-id <ID>    Unique id of a game.
        --test-env        Use the mod.io test environment
    -h, --help            Prints help information
```

### modiom subscriptions add

```
$ modiom subs add --help
modiom-subscriptions-add

USAGE:
    modiom subscriptions add [OPTIONS] <GAME> <MOD>

OPTIONS:
        --test-env    Use the mod.io test environment
    -h, --help        Prints help information

ARGS:
    <GAME>    Unique id of a game.
    <MOD>     Unique id of a mod.
```

### modiom subscriptions remove

```
$ modiom subs rm --help
modiom-subscriptions-remove

USAGE:
    modiom subscriptions remove [OPTIONS] <GAME> <MOD>

OPTIONS:
        --test-env    Use the mod.io test environment
    -h, --help        Prints help information

ARGS:
    <GAME>    Unique id of a game.
    <MOD>     Unique id of a mod.
```

### modiom download

```
$ modiom download --help
modiom-download

USAGE:
    modiom download [OPTIONS] --game-id <ID> --mod-id <ID>... [--] [DEST]

OPTIONS:
        --game-id <ID>
        --mod-id <ID>...
        --with-dependencies
        --test-env             use the test environment
    -h, --help                 Prints help information

ARGS:
    <DEST>
```

### modiom upload

```
$ modiom upload --help
modiom-upload
Upload new files

USAGE:
    modiom upload [OPTIONS] <GAME> <MOD> <FILE>

OPTIONS:
        --filename <NAME>          Overwrite the filename.
        --version <VERSION>        Version of this file release.
        --changelog <CHANGELOG>    Changelog of this release.
        --not-active               When this flag is enabled, the upload will not be labeled as current release.
        --metadata-blob <BLOB>
        --checksum                 Calculate the checksum before uploading.
        --test-env                 Use the mod.io test environment
    -h, --help                     Prints help information

ARGS:
    <GAME>    Unique id of the game.
    <MOD>     Unique id of the mod.
    <FILE>    Zip file to upload.
```

## The Modio Manifest Format

The `Modio.toml` file

### The `[game]` section

```toml
[game]
# id = (int|string)
id = "gametwo" # the name_id of the game or alternative its id
```

#### The `with-dependencies` field (optional)

This field specifies globally whether dependencies of mods are downloaded.
If you don't specify the field, it will default to `false`.

```toml
[game]
# ...
with-dependencies = true
```

### Specifying mods

```toml
[mods]
mod1 = 1
mod2 = "mod-two"
mod3 = { id = "mod-three" }

[mods.mod4]
id = 4
with-dependencies = true
```

#### The `with-dependencies` field (optional)

This field specifies whether dependencies of the mod are downloaded and overrides the global setting.
If you don't specify the field, it will default to `false`.

#### The `file` field (optional)

This field specifies the downloaded file id.

```toml
[mods.mod1]
id = "mod-one"
file = 34
```

#### The `version` field (optional)

This field specifies the downloaded version.

```toml
[mods.mod1]
id = "mod-one"
version = "1.2"
```
