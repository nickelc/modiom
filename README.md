<a href="https://mod.io"><img src="https://github.com/nickelc/modiom/raw/master/header.png" alt="mod.io" width="320"/></a>

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
modiom 0.3.0
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
Usage: modiom login [OPTIONS] [api-key] [token]

Arguments:
  [api-key]
  [token]

Options:
      --test-env  Use the mod.io test environment
  -h, --help      Print help information
```

### modiom search

```
$ modiom search --help
Search game or mods.

Usage: modiom search [OPTIONS] [FULLTEXT]

Arguments:
  [FULLTEXT]

Options:
      --game-id <ID>     When specified, modiom will search for mods of the game.
      --id <ID>          Specify the id of the game or mod.
      --name <VALUE>
      --name-id <VALUE>
      --expr <EXPR>
      --test-env         Use the mod.io test environment
  -h, --help             Print help information
```

### modiom info

```
$ modiom info --help
Show information of mods

Usage: modiom info [OPTIONS] <GAME> <MOD>

Arguments:
  <GAME>  Unique id of a game.
  <MOD>   Unique id of a mod.

Options:
      --files     List all files.
      --stats     Show the statistics.
      --test-env  Use the mod.io test environment
  -h, --help      Print help information
```

### modiom subscriptions

```
$ modiom subs --help
Show information of subscriptions

Usage: modiom subscriptions [OPTIONS] <COMMAND>

Commands:
  list
  add
  remove
  help    Print this message or the help of the given subcommand(s)

Options:
      --test-env  Use the mod.io test environment
  -h, --help      Print help information
```

### modiom subscriptions list

```
$ modiom subs list --help
Usage: modiom subscriptions list [OPTIONS]

Options:
      --game-id <ID>  Unique id of a game.
      --test-env      Use the mod.io test environment
  -h, --help          Print help information
```

### modiom subscriptions add

```
$ modiom subs add --help
Usage: modiom subscriptions add [OPTIONS] <GAME> <MOD>

Arguments:
  <GAME>  Unique id of a game.
  <MOD>   Unique id of a mod.

Options:
      --test-env  Use the mod.io test environment
  -h, --help      Print help information
```

### modiom subscriptions remove

```
$ modiom subs rm --help
Usage: modiom subscriptions remove [OPTIONS] <GAME> <MOD>

Arguments:
  <GAME>  Unique id of a game.
  <MOD>   Unique id of a mod.

Options:
      --test-env  Use the mod.io test environment
  -h, --help      Print help information
```

### modiom download

```
$ modiom download --help
Download mod files

Usage: modiom download [OPTIONS] --game-id <ID> --mod-id <ID> [DEST]

Arguments:
  [DEST]  Save files to DEST

Options:
      --game-id <ID>  Specify a game id
      --mod-id <ID>   Specify a mod id
      --test-env      Use the mod.io test environment
  -h, --help          Print help information
```

### modiom upload

```
$ modiom upload --help
Upload new files

Usage: modiom upload [OPTIONS] <GAME> <MOD> <FILE>

Arguments:
  <GAME>  Unique id of the game.
  <MOD>   Unique id of the mod.
  <FILE>  Zip file to upload.

Options:
      --filename <NAME>        Overwrite the filename.
      --version <VERSION>      Version of this file release.
      --changelog <CHANGELOG>  Changelog of this release.
      --not-active             The uploaded file will not be labeled as current release.
      --metadata-blob <BLOB>
      --checksum               Calculate the checksum before uploading.
      --test-env               Use the mod.io test environment
  -h, --help                   Print help information
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
