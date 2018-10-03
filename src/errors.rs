use std::io;

use cfg;
use clap;
use modio::Error as ModioError;

pub type ModiomResult<T> = Result<T, Error>;
pub type CliResult = Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    Clap(clap::Error),
    Config(cfg::ConfigError),
    Io(io::Error),
    Modio(ModioError),
    Message(String),
}

impl From<clap::Error> for Error {
    fn from(err: clap::Error) -> Error {
        Error::Clap(err)
    }
}

impl From<cfg::ConfigError> for Error {
    fn from(err: cfg::ConfigError) -> Error {
        Error::Config(err)
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<ModioError> for Error {
    fn from(err: ModioError) -> Error {
        Error::Modio(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Message(err)
    }
}
