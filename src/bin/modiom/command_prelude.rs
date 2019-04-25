use std::fs;
use std::path::PathBuf;

use clap::{self, SubCommand};

pub use clap::{value_t, values_t};
pub use clap::{AppSettings, Arg, ArgGroup, ArgMatches};
pub use modiom::config::Config;
pub use modiom::errors::{CliResult, Error, ModiomResult};
use modiom::utils::find_manifest_for_wd;
pub use prettytable::{cell, row, table};

pub type App = clap::App<'static, 'static>;

pub fn opt(name: &'static str, help: &'static str) -> Arg<'static, 'static> {
    Arg::with_name(name).long(name).help(help)
}

pub fn subcommand(name: &'static str) -> App {
    SubCommand::with_name(name).settings(&[
        AppSettings::UnifiedHelpMessage,
        AppSettings::DeriveDisplayOrder,
        AppSettings::DontCollapseArgsInUsage,
    ])
}

#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub fn validate_is_file(value: String) -> Result<(), String> {
    if !PathBuf::from(value).is_file() {
        return Err(String::from("Path is not a file."));
    }
    Ok(())
}

#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub fn validate_path_exists(value: String) -> Result<(), String> {
    if !PathBuf::from(value).exists() {
        return Err(String::from("Path does not exist."));
    }
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn validate_is_zip(value: String) -> Result<(), String> {
    if !PathBuf::from(&value).is_file() && value.ends_with(".zip") {
        return Err(String::from("File is not a zip."));
    }
    Ok(())
}

#[allow(clippy::needless_pass_by_value)]
pub fn validate_u32(value: String) -> Result<(), String> {
    match value.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    }
}

pub trait AppExt: Sized {
    fn _arg(self, arg: Arg<'static, 'static>) -> Self;

    fn arg_manifest_path(self) -> Self {
        self._arg(opt("manifest-path", "Path to Modio.toml").value_name("PATH"))
    }
}

impl AppExt for App {
    fn _arg(self, arg: Arg<'static, 'static>) -> Self {
        self.arg(arg)
    }
}

pub trait ArgMatchesExt {
    fn is_test_env(&self) -> bool {
        self._is_present("test-env")
    }

    fn _is_present(&self, _: &str) -> bool;

    fn _value_of(&self, name: &str) -> Option<&str>;

    fn value_of_path(&self, name: &str) -> Option<PathBuf> {
        self._value_of(name).map(PathBuf::from)
    }

    fn root_manifest(&self, config: &Config) -> ModiomResult<PathBuf>;
}

impl<'a> ArgMatchesExt for ArgMatches<'a> {
    fn _is_present(&self, name: &str) -> bool {
        self.is_present(name)
    }

    fn _value_of(&self, name: &str) -> Option<&str> {
        self.value_of(name)
    }

    fn root_manifest(&self, config: &Config) -> ModiomResult<PathBuf> {
        if let Some(path) = self.value_of_path("manifest-path") {
            if !path.ends_with("Modio.toml") {
                return Err(Error::Message(
                    "the manifest-path must be a path to a Modio.toml file".into(),
                ));
            }
            if fs::metadata(&path).is_err() {
                return Err(Error::Message(format!(
                    "manifest-path `{}` does not exist",
                    path.display(),
                )));
            }
            return Ok(path);
        }
        find_manifest_for_wd(config.cwd())
    }
}
