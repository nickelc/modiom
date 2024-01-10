use std::collections::BTreeMap;

use futures::TryStreamExt;
use prettytable::{format, Table};
use tokio::runtime::Runtime;

use modio::filter::prelude::*;
use modio::types::id;
use modio::types::mods::Mod;
use modio::user::filters::subscriptions::*;
use modiom::config::Config;

use crate::command_prelude::*;

type Subs = BTreeMap<id::GameId, Vec<Mod>>;

pub fn cli() -> Command {
    Command::new("subscriptions")
        .about("Show information of subscriptions")
        .alias("subs")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("list").arg(opt("game-id", "Unique id of a game.").value_name("ID")),
        )
        .subcommand(
            Command::new("add")
                .arg(
                    Arg::new("game")
                        .help("Unique id of a game.")
                        .value_name("GAME")
                        .required(true)
                        .value_parser(value_parser!(u32)),
                )
                .arg(
                    Arg::new("mod")
                        .help("Unique id of a mod.")
                        .value_name("MOD")
                        .required(true)
                        .value_parser(value_parser!(u32)),
                ),
        )
        .subcommand(
            Command::new("remove")
                .alias("rm")
                .arg(
                    Arg::new("game")
                        .help("Unique id of a game.")
                        .value_name("GAME")
                        .required(true)
                        .value_parser(value_parser!(u32)),
                )
                .arg(
                    Arg::new("mod")
                        .help("Unique id of a mod.")
                        .value_name("MOD")
                        .required(true)
                        .value_parser(value_parser!(u32)),
                ),
        )
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    match args.subcommand() {
        Some(("list", matches)) => list_subs(config, matches),
        Some(("add", matches)) => subscribe(config, matches),
        Some(("remove", matches)) => unsubscribe(config, matches),
        _ => unreachable!(),
    }
}

fn list_subs(config: &Config, args: &ArgMatches) -> CliResult {
    let game_id = args.get_one::<id::GameId>("game-id");

    let rt = Runtime::new()?;
    let m = client(config)?;

    let filter = if let Some(game_id) = game_id {
        GameId::eq(game_id)
    } else {
        Default::default()
    };

    let task = async {
        let st = m.user().subscriptions(filter).iter().await?;
        let mut subs = st
            .try_fold(Subs::new(), |mut subs, m| async {
                subs.entry(m.game_id).or_default().push(m);
                Ok(subs)
            })
            .await?;
        let filter = Id::_in(subs.keys().collect::<Vec<_>>());
        m.games()
            .search(filter)
            .iter()
            .await?
            .map_ok(move |g| subs.remove(&g.id).map(|mods| (g, mods)))
            .try_filter_map(|s| async { Ok(s) })
            .try_collect::<Vec<_>>()
            .await
    };

    let subs = rt.block_on(task)?;
    for (game, mods) in subs {
        let mut output = Table::new();
        output.set_format(*format::consts::FORMAT_CLEAN);
        output.set_titles(row![
            b -> "Id",
            b -> "Name-Id",
            b -> "Name",
        ]);
        for m in mods {
            output.add_row(row![m.id, m.name_id, m.name]);
        }
        println!("Subscriptions: {}. {}", game.id, game.name);
        output.printstd();
    }
    Ok(())
}

fn subscribe(config: &Config, args: &ArgMatches) -> CliResult {
    let game_id = *args.get_one("game").expect("required arg");
    let mod_id = *args.get_one("mod").expect("required arg");

    let rt = Runtime::new()?;
    let m = client(config)?;

    rt.block_on(m.mod_(game_id, mod_id).subscribe())?;
    Ok(())
}

fn unsubscribe(config: &Config, args: &ArgMatches) -> CliResult {
    let game_id = *args.get_one("game").expect("required arg");
    let mod_id = *args.get_one("mod").expect("required arg");

    let rt = Runtime::new()?;
    let m = client(config)?;

    rt.block_on(m.mod_(game_id, mod_id).unsubscribe())?;
    Ok(())
}
