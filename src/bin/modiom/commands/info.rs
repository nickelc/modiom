use prettytable::format;
use textwrap::fill;
use tokio::runtime::Runtime;

use command_prelude::*;

use modio::auth::Credentials;
use modio::Modio;
use modiom::config::Config;

pub fn cli() -> App {
    subcommand("info")
        .about("Show information of mods")
        .arg(
            Arg::with_name("game")
                .value_name("GAME")
                .required(true)
                .validator(validate_u32),
        ).arg(
            Arg::with_name("mod")
                .value_name("MOD")
                .required(true)
                .validator(validate_u32),
        )
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    let game_id = value_t!(args, "game", u32)?;
    let mod_id = value_t!(args, "mod", u32)?;

    if let Ok(Some(token)) = config.auth_token() {
        let mut rt = Runtime::new()?;
        let modio = Modio::host(config.host(), "modiom", Credentials::Token(token));

        match rt.block_on(modio.mod_(game_id, mod_id).get()) {
            Ok(m) => {
                let tags = m
                    .tags
                    .iter()
                    .map(|t| t.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                let mut mt = table!(
                    [b -> "Id", m.id],
                    [b -> "Name-Id", m.name_id],
                    [b -> "Name", m.name],
                    [b -> "Summary", fill(&m.summary, 60)],
                    [b -> "Profile", m.profile_url],
                    [b -> "Homepage", m.homepage_url.map(|u| u.to_string()).unwrap_or_else(String::new)],
                    [b -> "Tags", tags]
                );
                mt.set_format(*format::consts::FORMAT_CLEAN);
                if let Some(file) = m.modfile {
                    mt.add_empty_row();
                    mt.add_row(row![bH2 -> "File"]);
                    mt.add_row(row![b -> "Id", file.id]);
                    mt.add_row(row![b -> "Filename", file.filename]);
                    mt.add_row(row![b -> "Version", file.version.unwrap_or_else(String::new)]);
                    mt.add_row(row![b -> "Download", file.download.binary_url]);
                    mt.add_row(row![b -> "Size", file.filesize]);
                    mt.add_row(row![b -> "MD5", file.filehash.md5]);
                }
                mt.printstd();
            }
            Err(e) => println!("{}", e),
        };
    }
    Ok(())
}
