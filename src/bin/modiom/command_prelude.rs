use std::any::Any;
use std::borrow::Cow;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub use clap::builder::{
    Arg, ArgAction, Command, PathBufValueParser, TypedValueParser, ValueParser,
};
pub use clap::{value_parser, ArgMatches};
pub use modiom::config::Config;
pub use modiom::{CliResult, Result};
pub use prettytable::{row, table};

use modiom::utils::find_manifest_for_wd;

pub fn client(config: &Config) -> Result<modio::Modio> {
    let token = config
        .auth_token()?
        .ok_or("authentication token required")?;

    let client = modio::Modio::builder(token)
        .host(config.host())
        .user_agent("modiom")
        .build()?;
    Ok(client)
}

pub fn opt(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).help(help)
}

pub trait CommandExt: Sized {
    fn _arg(self, arg: Arg) -> Self;

    fn arg_manifest_path(self) -> Self {
        self._arg(
            opt("manifest-path", "Path to Modio.toml")
                .value_name("PATH")
                .value_parser(ValueParser::path_buf()),
        )
    }
}

impl CommandExt for Command {
    fn _arg(self, arg: Arg) -> Self {
        self.arg(arg)
    }
}

pub trait ArgMatchesExt {
    fn _get_flag(&self, _: &str) -> bool;

    fn _get_one<T: Any + Clone + Send + Sync + 'static>(&self, _: &str) -> Option<&T>;

    fn is_test_env(&self) -> bool {
        self._get_flag("test-env")
    }

    fn root_manifest(&self, config: &Config) -> io::Result<Cow<Path>> {
        if let Some(path) = self._get_one::<PathBuf>("manifest-path") {
            if !path.ends_with("Modio.toml") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "the manifest-path must be a path to a Modio.toml file",
                ));
            }
            if fs::metadata(path).is_err() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("manifest-path `{}` does not exist", path.display()),
                ));
            }
            return Ok(path.into());
        }
        find_manifest_for_wd(config.cwd()).map(Cow::from)
    }

    fn get_string(&self, id: &str) -> Option<&String> {
        self._get_one::<String>(id)
    }
}

impl ArgMatchesExt for ArgMatches {
    fn _get_flag(&self, id: &str) -> bool {
        self.get_flag(id)
    }

    fn _get_one<T: Any + Clone + Send + Sync + 'static>(&self, id: &str) -> Option<&T> {
        self.get_one(id)
    }
}
