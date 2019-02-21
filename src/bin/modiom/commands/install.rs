use std::path::Path;

use futures::{future, future::Either, Future as StdFuture};
use modio::filter::Operator;
use modio::games::GamesListOptions;
use modio::mods::ModsListOptions;
use modiom::errors::Error;
use modiom::manifest::{self, Identifier};
use tokio::fs::File;
use tokio::runtime::Runtime;

use crate::command_prelude::*;
use crate::progress::ProgressWrapper;

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

    let mut rt = Runtime::new()?;
    let modio = config.client()?;

    let game_id = match manifest.game.id {
        Identifier::Id(id) => Either::A(future::ok(id)),
        Identifier::NameId(ref id) => {
            let err = format_err!("no matching game named `{}` found", id);
            let mut opts = GamesListOptions::new();
            opts.name_id(Operator::Equals, id);
            Either::B(
                modio
                    .games()
                    .list(&opts)
                    .map_err(Error::from)
                    .and_then(|list| match list.first() {
                        Some(game) => Ok(game.id),
                        None => Err(err),
                    }),
            )
        }
    };
    let tasks = game_id.and_then(|game_id| {
        let tasks = manifest
            .mods
            .unwrap_or_default()
            .iter()
            .map(move |(_, m)| {
                let modio2 = modio.clone();
                let mut opts = ModsListOptions::new();
                let not_found = match m.id() {
                    Identifier::Id(id) => {
                        opts.id(Operator::Equals, id);
                        format_err!("mod with id `{}` not found", id)
                    }
                    Identifier::NameId(id) => {
                        opts.name_id(Operator::Equals, id);
                        format_err!("mod with name-id `{}` not found", id)
                    }
                };
                modio
                    .game(game_id)
                    .mods()
                    .list(&opts)
                    .map_err(Error::from)
                    .and_then(|mut list| match list.shift() {
                        Some(mod_) => Ok(mod_),
                        None => Err(not_found),
                    })
                    .and_then(move |mod_| match mod_.modfile {
                        Some(file) => Either::A(future::ok(file)),
                        None => Either::B(future::err(format_err!(
                            "mod `{}` has no primary file",
                            mod_.name_id
                        ))),
                    })
                    .and_then(|file| {
                        let dest = Path::new("");
                        File::create(dest.join(&file.filename))
                            .map_err(Error::from)
                            .join(future::ok(file))
                    })
                    .and_then(move |(out, file)| {
                        println!("Downloading: {}", file.download.binary_url);

                        let w = ProgressWrapper::new(out, file.filesize);
                        modio2
                            .download(file, w)
                            .map_err(Error::from)
                            .and_then(|(_n, mut w)| {
                                w.finish();
                                Ok(())
                            })
                    })
            })
            .collect::<Vec<_>>();
        future::join_all(tasks)
    });
    rt.block_on(tasks)?;

    Ok(())
}
