use std::io::{self, BufRead, Write};

use clap::Arg;
use modio::auth::Credentials;
use modio::Modio;
use tokio::runtime::Runtime;

use modiom::config::Config;
use modiom::errors::ModiomResult;

use crate::command_prelude::*;

pub fn cli() -> App {
    subcommand("login").arg(Arg::with_name("token"))
}

pub fn exec(config: &Config, args: &ArgMatches<'_>) -> CliResult {
    let token = match args.value_of("token") {
        Some(token) => Credentials::Token(token.to_string()),
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
            let m = Modio::host(config.host(), Credentials::ApiKey(api_key))?;

            let message = rt.block_on(m.auth().request_code(&email))?;
            println!("{}", message);

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

    config.save_credentials(token.to_string())?;
    Ok(())
}

fn prompt(prompt: &str) -> ModiomResult<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut buf = String::new();
    let input = io::stdin();
    input.lock().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}
