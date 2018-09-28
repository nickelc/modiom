use std::io;

use cfg;
use clap;
use modio::Error as ModioError;
use reqwest;

pub type ModiomResult<T> = Result<T, Error>;
pub type CliResult = Result<(), Error>;

#[derive(Debug)]
pub enum Error {
    Clap(clap::Error),
    Config(cfg::ConfigError),
    Io(io::Error),
    Modio(ModioError),
    Reqwest(reqwest::Error),
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
        match err {
            ModioError::Msg(msg) => Error::Message(msg),
            ModioError::Fault { error, .. } => {
                let mut msg = String::new();
                msg.push_str(&error.message);
                if let Some(errors) = error.errors {
                    msg.push('\n');
                    for (key, val) in errors {
                        msg.push_str(&format!("{}: {}", key, val));
                    }
                }
                Error::Message(msg)
            }
            e => Error::Modio(e),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Reqwest(err)
    }
}

impl From<String> for Error {
    fn from(err: String) -> Error {
        Error::Message(err)
    }
}
