mod command_prelude;
mod commands;

use crate::command_prelude::*;

fn main() -> CliResult {
    let args = Command::new("modiom")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands(commands::builtin())
        .arg(
            opt("test-env", "Use the mod.io test environment")
                .global(true)
                .action(ArgAction::SetTrue),
        )
        .try_get_matches()
        .unwrap_or_else(|e| e.exit());

    let mut config = Config::default()?;
    config.configure(args.is_test_env())?;

    match commands::exec(&config, &args) {
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
        Ok(()) => Ok(()),
    }
}
