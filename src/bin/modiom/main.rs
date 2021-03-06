use modiom::config::Config;
use modiom::CliResult;

mod command_prelude;
mod commands;
// mod progress;

use crate::command_prelude::*;

fn main() -> CliResult {
    let args = App::new("modiom")
        .settings(&[
            AppSettings::UnifiedHelpMessage,
            AppSettings::DeriveDisplayOrder,
            AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands,
        ])
        .subcommands(commands::builtin())
        .arg(opt("test-env", "Use the mod.io test environment").global(true))
        .get_matches_safe()
        .unwrap_or_else(|e| e.exit());

    let mut config = Config::default()?;
    config.configure(args.is_test_env())?;

    match commands::exec(&config, &args) {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        Ok(()) => Ok(()),
    }
}
