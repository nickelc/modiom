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
