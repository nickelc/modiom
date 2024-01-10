use std::path::Path;

use futures::{stream, TryStreamExt};
use modio::filter::prelude::*;
use modio::types::id::GameId;
use modiom::manifest::{self, Identifier};
use tokio::runtime::Runtime;

use crate::command_prelude::*;

pub fn cli() -> Command {
    Command::new("install").arg_manifest_path()
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    let path = args.root_manifest(config)?;
    let manifest = manifest::read(&path)?;

    match manifest.mods {
        Some(ref mods) if !mods.is_empty() => {}
        _ => return Err("no mods defined".into()),
    }

    let rt = Runtime::new()?;
    let modio = client(config)?;

    let tasks = async {
        let game_id = match manifest.game.id {
            Identifier::Id(id) => GameId::new(id),
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
        let mut tasks = vec![];
        for (_, m) in manifest.mods.unwrap_or_default() {
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
            tasks.push(async {
                let mod_ = modio.game(game_id).mods().search(filter).first().await?;
                match mod_ {
                    None => Err(not_found),
                    Some(m) if m.modfile.is_none() => {
                        Err(format!("mod `{}` has no primary file", m.name_id).into())
                    }
                    Some(m) => {
                        let file = m.modfile.unwrap();
                        println!("Downloading: {}", file.download.binary_url);
                        let out = Path::new(&file.filename).to_path_buf();
                        modio.download(file).await?.save_to_file(out).await?;
                        Ok(())
                    }
                }
            });
        }
        tasks
            .into_iter()
            .collect::<stream::FuturesUnordered<_>>()
            .try_collect::<Vec<()>>()
            .await
    };

    rt.block_on(tasks)?;

    Ok(())
}
