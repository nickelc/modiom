use std::fs::File;
use std::io;
use std::path::PathBuf;

use reqwest;
use tokio::runtime::Runtime;

use command_prelude::*;

use modio::auth::Credentials;
use modio::filter::Operator;
use modio::mods::ModsListOptions;
use modio::Modio;
use modiom::config::Config;
use progress::ProgressWrapper;

pub fn cli() -> App {
    subcommand("download")
        .arg(
            opt("game-id", "")
                .value_name("ID")
                .number_of_values(1)
                .required(true)
                .validator(validate_u32),
        ).arg(
            opt("mod-id", "")
                .value_name("ID")
                .multiple(true)
                .number_of_values(1)
                .required(true)
                .validator(validate_u32),
        ).arg(opt("with-dependencies", ""))
        .arg(Arg::with_name("dest").value_name("DEST"))
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    let game_id = value_t!(args, "game-id", u32)?;
    let mod_ids = values_t!(args, "mod-id", u32)?;
    let _with_deps = args.is_present("with-dependencies");
    let dest = value_t!(args, "dest", String)
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(PathBuf::new);

    if let Ok(Some(token)) = config.auth_token() {
        let mut rt = Runtime::new()?;
        let modio_ = Modio::host(
            config.host(),
            "modiom",
            Credentials::Token(token),
        );
        let mut opts = ModsListOptions::new();
        opts.id(Operator::In, mod_ids);

        println!("{:#?}", opts);

        let list = rt.block_on(modio_.game(game_id).mods().list(&opts));

        // println!("{:#?}", list);
        if let Ok(mods) = list {
            for m in mods {
                if let Some(file) = m.modfile {
                    println!(
                        "dl: {} to {}",
                        file.download.binary_url,
                        dest.join(file.filename.clone()).display()
                    );
                    let mut res = reqwest::get(file.download.binary_url)?;

                    let mut out = File::create(dest.join(file.filename))?;
                    let mut w = ProgressWrapper::new(out, file.filesize);
                    io::copy(&mut res, &mut w)?;
                    w.finish();
                    // if with_deps {
                    //     let deps_list = rt.block_on(modio_.mod_(game_id, m.id).dependencies().list());
                    //     if let Ok(deps) = deps_list {
                    //         for d in deps {
                    //             rt.
                    //         }
                    //     }
                    // }
                }
            }
        }
    }

    Ok(())
}
