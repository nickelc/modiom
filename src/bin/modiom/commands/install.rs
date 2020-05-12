use std::path::Path;

use futures::{future, TryFutureExt};
use futures::{stream, TryStreamExt};
use modio::filter::prelude::*;
use modiom::manifest::{self, Identifier};
use tokio::runtime::Runtime;

use crate::command_prelude::*;

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

    let mut rt = Runtime::new()?;
    let modio = client(config)?;

    let tasks = async {
        let game_id = match manifest.game.id {
            Identifier::Id(id) => id,
            Identifier::NameId(ref id) => {
                let filter = NameId::eq(id);
                let first = modio.games().search(filter).first().await?;
                if let Some(game) = first {
                    game.id
                } else {
                    return Err(format!("no matching game named `{}` found", id).into());
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
                let not_found: Box<dyn std::error::Error> = match m.id() {
                    Identifier::Id(id) => {
                        filter = Id::eq(id);
                        format!("mod with id `{}` not found", id).into()
                    }
                    Identifier::NameId(id) => {
                        filter = NameId::eq(id);
                        format!("mod with name-id `{}` not found", id).into()
                    }
                };
                modio
                    .game(game_id)
                    .mods()
                    .search(filter)
                    .first_page()
                    .map_err(Into::into)
                    .and_then(|mut list| match list.is_empty() {
                        false => future::ok(list.remove(0)),
                        true => future::err(not_found),
                    })
                    .and_then(move |mod_| match mod_.modfile {
                        Some(file) => future::ok(file),
                        None => future::err(
                            format!("mod `{}` has no primary file", mod_.name_id).into(),
                        ),
                    })
                    .and_then(move |file| {
                        println!("Downloading: {}", file.download.binary_url);
                        let out = Path::new(&file.filename).to_path_buf();
                        modio2.download(file).save_to_file(out).map_err(Into::into)
                    })
            })
            .collect::<stream::FuturesUnordered<_>>();

        tasks.try_collect::<Vec<_>>().await
    };

    rt.block_on(tasks)?;

    Ok(())
}
