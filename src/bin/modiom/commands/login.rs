use std::io::{self, BufRead, Write};

use modio::auth::Credentials;
use modio::Modio;
use tokio::runtime::Runtime;

use crate::command_prelude::*;

pub fn cli() -> Command {
    Command::new("login")
        .arg(Arg::new("api-key"))
        .arg(Arg::new("token"))
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    let api_key = args.get_string("api-key");
    let token = args.get_string("token");

    let token = match (api_key, token) {
        (Some(api_key), Some(token)) => Credentials::with_token(api_key, token),
        (api_key, _) => {
            let api_key = if let Some(api_key) = api_key {
                api_key.into()
            } else {
                let url = if args.is_test_env() {
                    "https://test.mod.io/apikey"
                } else {
                    "https://mod.io/apikey"
                };
                println!("Please visit {url} and paste the API key below");

                prompt("Enter api key: ")?
            };
            let email = prompt("Enter email: ")?;

            let rt = Runtime::new()?;
            let m = Modio::host(config.host(), Credentials::new(api_key))?;

            rt.block_on(m.auth().request_code(&email))?;
            println!("Authentication code request was successful.");

            loop {
                let code = prompt("Enter security code: ")?;
                match rt.block_on(m.auth().security_code(&code)) {
                    Ok(token) => break token,
                    Err(err) => println!("{err}"),
                };
            }
        }
    };

    if let Ok(Some(old_token)) = config.auth_token() {
        if token == old_token {
            return Ok(());
        }
    }

    if let Credentials {
        api_key,
        token: Some(token),
    } = token
    {
        config.save_credentials(api_key, token.value)?;
    }
    Ok(())
}

fn prompt(prompt: &str) -> io::Result<String> {
    print!("{prompt}");
    io::stdout().flush()?;
    let mut buf = String::new();
    let input = io::stdin();
    input.lock().read_line(&mut buf)?;
    Ok(buf.trim().to_string())
}
