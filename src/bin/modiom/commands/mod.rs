use modiom::config::Config;

use crate::command_prelude::*;

pub fn builtin() -> Vec<App> {
    vec![
        login::cli(),
        info::cli(),
        search::cli(),
        download::cli(),
        upload::cli(),
    ]
}

pub fn exec(cfg: &Config, args: &ArgMatches) -> CliResult {
    match args.subcommand() {
        ("login", Some(matches)) => login::exec(cfg, matches),
        ("info", Some(matches)) => info::exec(cfg, matches),
        ("search", Some(matches)) => search::exec(cfg, matches),
        ("download", Some(matches)) => download::exec(cfg, matches),
        ("upload", Some(matches)) => upload::exec(cfg, matches),
        _ => unreachable!(),
    }
}

mod download;
mod expr;
mod info;
mod login;
mod search;
mod upload;
