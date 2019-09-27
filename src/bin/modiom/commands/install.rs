use std::path::Path;

use futures::{future, TryFutureExt};
use modio::filter::prelude::*;
use modiom::errors::Error;
use modiom::manifest::{self, Identifier};
use tokio::runtime::Runtime;

use crate::command_prelude::*;

macro_rules! format_err {
    ($($arg:tt)*) => { Error::Message(format!($($arg)*)) };
}

pub fn cli() -> App {
    subcommand("install").arg_manifest_path()
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    let path = args.root_manifest(config)?;
    let manifest = manifest::read(&path)?;

    match manifest.mods {
        Some(ref mods) if !mods.is_empty() => {}
        _ => return Err("no mods defined".into()),
    }

    let rt = Runtime::new()?;
    let modio = config.client()?;

    let tasks = async {
        let game_id = match manifest.game.id {
            Identifier::Id(id) => id,
            Identifier::NameId(ref id) => {
                let filter = NameId::eq(id);
                let list = modio.games().list(filter).await?;
                if let Some(game) = list.first() {
                    game.id
                } else {
                    return Err(format_err!("no matching game named `{}` found", id));
                }
            }
        };
        let tasks = manifest
            .mods
            .unwrap_or_default()
            .iter()
            .map(move |(_, m)| {
                let modio2 = modio.clone();
                let filter;
                let not_found = match m.id() {
                    Identifier::Id(id) => {
                        filter = Id::eq(id);
                        format_err!("mod with id `{}` not found", id)
                    }
                    Identifier::NameId(id) => {
                        filter = NameId::eq(id);
                        format_err!("mod with name-id `{}` not found", id)
                    }
                };
                modio
                    .game(game_id)
                    .mods()
                    .list(filter)
                    .map_err(Error::from)
                    .and_then(|mut list| match list.shift() {
                        Some(mod_) => future::ok(mod_),
                        None => future::err(not_found),
                    })
                    .and_then(move |mod_| match mod_.modfile {
                        Some(file) => future::ok(file),
                        None => {
                            future::err(format_err!("mod `{}` has no primary file", mod_.name_id))
                        }
                    })
                    .and_then(move |file| {
                        println!("Downloading: {}", file.download.binary_url);
                        let out = Path::new(&file.filename).to_path_buf();
                        modio2.download(file).save_to_file(out).map_err(Error::from)
                    })
            })
            .collect::<Vec<_>>();
        future::try_join_all(tasks).await
    };

    rt.block_on(tasks)?;

    Ok(())
}
