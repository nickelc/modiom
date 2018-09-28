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
