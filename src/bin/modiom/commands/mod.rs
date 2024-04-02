use crate::command_prelude::*;

pub fn builtin() -> Vec<Command> {
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

pub fn exec(cfg: &Config, args: &ArgMatches) -> CliResult {
    match args.subcommand() {
        Some(("login", matches)) => login::exec(cfg, matches),
        Some(("info", matches)) => info::exec(cfg, matches),
        Some(("search", matches)) => search::exec(cfg, matches),
        Some(("subscriptions", matches)) => subs::exec(cfg, matches),
        Some(("download", matches)) => download::exec(cfg, matches),
        Some(("upload", matches)) => upload::exec(cfg, matches),
        Some(("install", matches)) => install::exec(cfg, matches),
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
