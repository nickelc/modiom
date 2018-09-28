use command_prelude::*;
use modiom::config::Config;

pub fn builtin() -> Vec<App> {
    vec![login::cli(), search::cli(), download::cli()]
}

pub fn exec(cfg: &Config, args: &ArgMatches) -> CliResult {
    match args.subcommand() {
        ("login", Some(matches)) => login::exec(cfg, matches),
        ("search", Some(matches)) => search::exec(cfg, matches),
        ("download", Some(matches)) => download::exec(cfg, matches),
        _ => unreachable!(),
    }
}

mod download;
mod expr;
mod login;
mod search;
