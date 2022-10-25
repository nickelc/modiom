use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;

use tokio::runtime::Runtime;

use modio::filter::prelude::*;
use modiom::config::Config;

use crate::command_prelude::*;

pub fn cli() -> Command {
    Command::new("download")
        .about("Download mod files")
        .arg(
            opt("game-id", "Specify a game id")
                .value_name("ID")
                .num_args(1)
                .required(true)
                .value_parser(value_parser!(u32)),
        )
        .arg(
            opt("mod-id", "Specify a mod id")
                .value_name("ID")
                .num_args(1)
                .required(true)
                .value_parser(value_parser!(u32))
                .action(ArgAction::Append),
        )
        //.arg(opt("with-dependencies", ""))
        .arg(
            Arg::new("dest")
                .help("Save files to DEST")
                .value_name("DEST")
                .value_parser(ValueParser::path_buf()),
        )
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    let game_id = *args.get_one("game-id").expect("required arg");
    let mod_ids = args.get_many::<u32>("mod-id").expect("required arg");
    let _with_deps = args.get_flag("with-dependencies");
    let dest = args
        .get_one::<PathBuf>("dest")
        .map(Cow::from)
        .unwrap_or_default();

    let rt = Runtime::new()?;
    let modio_ = client(config)?;

    let mod_ids = mod_ids.copied().collect::<Vec<_>>();
    let mut missing_mods: HashSet<u32> = HashSet::new();
    missing_mods.extend(&mod_ids);

    let filter = Id::_in(mod_ids);

    let list = rt.block_on(modio_.game(game_id).mods().search(filter).first_page());

    if let Ok(mods) = list {
        for m in mods {
            if let Some(file) = m.modfile {
                println!("Downloading: {}", file.download.binary_url);

                let out = dest.join(&file.filename);
                rt.block_on(modio_.download(file).save_to_file(out))?;
            }
            missing_mods.remove(&m.id);
        }
    }
    for mm in missing_mods {
        println!("Mod.id: {} does not exist or has no primary file. ", mm);
    }

    Ok(())
}
