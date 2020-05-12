use std::io::{self, BufRead, Write};

use clap::Arg;
use modio::auth::Credentials;
use modio::Modio;
use tokio::runtime::Runtime;

use modiom::config::Config;

use crate::command_prelude::*;

pub fn cli() -> App {
    subcommand("login").arg(Arg::with_name("token"))
}

pub fn exec(config: &Config, args: &ArgMatches<'_>) -> CliResult {
    let token = match args.value_of("token") {
        // FIXME
        Some(token) => Credentials::with_token("FIXME", token),
        None => {
            let url = if args.is_test_env() {
                "https://test.mod.io/apikey"
            } else {
                "https://mod.io/apikey"
            };
            println!("Please visit {} and paste the API key below", url);

            let api_key = prompt("Enter api key: ")?;
            let email = prompt("Enter email: ")?;

            let mut rt = Runtime::new()?;
            let m = Modio::host(config.host(), Credentials::new(api_key))?;

            rt.block_on(m.auth().request_code(&email))?;
            println!("Authentication code request was successful.");

            loop {
                let code = prompt("Enter security code: ")?;
                match rt.block_on(m.auth().security_code(&code)) {
                    Ok(token) => break token,
                    Err(err) => println!("{}", err),
                };
            }
        }
    };

    if let Ok(Some(old_token)) = config.auth_token() {
        if token == old_token {
            return Ok(());
        }
    }

    // FIXME
    config.save_credentials(token.token.unwrap().value)?;
    Ok(())
}

fn prompt(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buf = String::new();
    let input = io::stdin();
    input.lock().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}
