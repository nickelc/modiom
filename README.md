<a href="https://mod.io"><img src="https://static.mod.io/v1/images/branding/modio-color-dark.svg" alt="mod.io" width="400"/></a>

# modiom

modiom is a command line tool for [mod.io](https://mod.io) to search, download and update mods for games without builtin support.

## Building

modiom is written in Rust, so you'll need to grab a
[Rust installation](https://www.rust-lang.org/) in order to compile it.
Building is easy:

```
$ git clone https://github.com/nickelc/modiom.git
$ cd hpk
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
    modiom login [OPTIONS] [token]

OPTIONS:
        --test-env    use the test environment
    -h, --help        Prints help information

ARGS:
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
