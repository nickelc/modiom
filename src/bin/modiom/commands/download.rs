use std::collections::HashSet;
use std::fs::File;
use std::path::PathBuf;

use tokio::runtime::Runtime;

use modio::auth::Credentials;
use modio::filter::Operator;
use modio::mods::ModsListOptions;
use modio::Modio;
use modiom::config::Config;

use crate::command_prelude::*;
use crate::progress::ProgressWrapper;

pub fn cli() -> App {
    subcommand("download")
        .about("Download mod files")
        .arg(
            opt("game-id", "Specify a game id")
                .value_name("ID")
                .number_of_values(1)
                .required(true)
                .validator(validate_u32),
        )
        .arg(
            opt("mod-id", "Specify a mod id")
                .value_name("ID")
                .multiple(true)
                .number_of_values(1)
                .required(true)
                .validator(validate_u32),
        )
        //.arg(opt("with-dependencies", ""))
        .arg(
            Arg::with_name("dest")
                .help("Save files to DEST")
                .value_name("DEST"),
        )
}

pub fn exec(config: &Config, args: &ArgMatches<'_>) -> CliResult {
    let game_id = value_t!(args, "game-id", u32)?;
    let mod_ids = values_t!(args, "mod-id", u32)?;
    let _with_deps = args.is_present("with-dependencies");
    let dest = value_t!(args, "dest", String)
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(PathBuf::new);

    if let Ok(Some(token)) = config.auth_token() {
        let mut rt = Runtime::new()?;
        let modio_ = Modio::host(config.host(), "modiom", Credentials::Token(token));

        let mut missing_mods: HashSet<u32> = HashSet::new();
        missing_mods.extend(&mod_ids);

        let mut opts = ModsListOptions::new();
        opts.id(Operator::In, mod_ids);

        let list = rt.block_on(modio_.game(game_id).mods().list(&opts));

        if let Ok(mods) = list {
            for m in mods {
                if let Some(file) = m.modfile {
                    println!("Downloading: {}", file.download.binary_url);

                    let out = File::create(dest.join(&file.filename))?;
                    let w = ProgressWrapper::new(out, file.filesize);
                    let (_n, mut w) = rt.block_on(modio_.download(file, w))?;
                    w.finish();
                }
                missing_mods.remove(&m.id);
            }
        }
        for mm in missing_mods {
            println!("Mod.id: {} does not exist or has no primary file. ", mm);
        }
    }

    Ok(())
}
