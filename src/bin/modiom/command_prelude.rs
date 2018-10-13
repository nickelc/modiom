use std::path::PathBuf;

use clap::{self, SubCommand};

pub use clap::{AppSettings, Arg, ArgGroup, ArgMatches};
pub use modiom::errors::CliResult;

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
pub fn validate_is_file(value: String) -> Result<(), String> {
    if !PathBuf::from(value).is_file() {
        return Err(String::from("Path is not a file."));
    }
    Ok(())
}

#[allow(dead_code)]
pub fn validate_path_exists(value: String) -> Result<(), String> {
    if !PathBuf::from(value).exists() {
        return Err(String::from("Path does not exist."));
    }
    Ok(())
}

pub fn validate_is_zip(value: String) -> Result<(), String> {
    if !PathBuf::from(&value).is_file() && value.ends_with(".zip") {
        return Err(String::from("File is not a zip."));
    }
    Ok(())
}

pub fn validate_u32(value: String) -> Result<(), String> {
    match value.parse::<u32>() {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}", e)),
    }
}

pub trait ArgMatchesExt {
    fn is_test_env(&self) -> bool {
        self._is_present("test-env")
    }

    fn _is_present(&self, &str) -> bool;
}

impl<'a> ArgMatchesExt for ArgMatches<'a> {
    fn _is_present(&self, name: &str) -> bool {
        self.is_present(name)
    }
}
