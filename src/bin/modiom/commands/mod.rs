use modiom::config::Config;

use crate::command_prelude::*;

pub fn builtin() -> Vec<App> {
    vec![
        login::cli(),
        info::cli(),
        search::cli(),
        subs::cli(),
        download::cli(),
        upload::cli(),
        install::cli(),
    ]
}

pub fn exec(cfg: &Config, args: &ArgMatches<'_>) -> CliResult {
    match args.subcommand() {
        ("login", Some(matches)) => login::exec(cfg, matches),
        ("info", Some(matches)) => info::exec(cfg, matches),
        ("search", Some(matches)) => search::exec(cfg, matches),
        ("subscriptions", Some(matches)) => subs::exec(cfg, matches),
        ("download", Some(matches)) => download::exec(cfg, matches),
        ("upload", Some(matches)) => upload::exec(cfg, matches),
        ("install", Some(matches)) => install::exec(cfg, matches),
        _ => unreachable!(),
    }
}

mod download;
mod expr;
mod info;
mod install;
mod login;
mod search;
mod subs;
mod upload;
