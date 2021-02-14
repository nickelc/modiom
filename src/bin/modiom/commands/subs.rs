use std::collections::BTreeMap;

use futures::TryStreamExt;
use prettytable::{format, Table};
use tokio::runtime::Runtime;

use modio::filter::prelude::*;
use modio::mods::Mod;
use modio::user::filters::subscriptions::*;
use modiom::config::Config;

use crate::command_prelude::*;

type Subs = BTreeMap<u32, Vec<Mod>>;

pub fn cli() -> App {
    subcommand("subscriptions")
        .settings(&[
            AppSettings::UnifiedHelpMessage,
            AppSettings::DeriveDisplayOrder,
            AppSettings::SubcommandRequiredElseHelp,
            AppSettings::VersionlessSubcommands,
        ])
        .about("Show information of subscriptions")
        .alias("subs")
        .subcommand(subcommand("list").arg(opt("game-id", "Unique id of a game.").value_name("ID")))
        .subcommand(
            subcommand("add")
                .arg(
                    Arg::with_name("game")
                        .help("Unique id of a game.")
                        .value_name("GAME")
                        .required(true)
                        .validator(validate_u32),
                )
                .arg(
                    Arg::with_name("mod")
                        .help("Unique id of a mod.")
                        .value_name("MOD")
                        .required(true)
                        .validator(validate_u32),
                ),
        )
        .subcommand(
            subcommand("remove")
                .alias("rm")
                .arg(
                    Arg::with_name("game")
                        .help("Unique id of a game.")
                        .value_name("GAME")
                        .required(true)
                        .validator(validate_u32),
                )
                .arg(
                    Arg::with_name("mod")
                        .help("Unique id of a mod.")
                        .value_name("MOD")
                        .required(true)
                        .validator(validate_u32),
                ),
        )
}

pub fn exec(config: &Config, args: &ArgMatches<'_>) -> CliResult {
    match args.subcommand() {
        ("list", Some(matches)) => list_subs(config, matches),
        ("add", Some(matches)) => subscribe(config, matches),
        ("remove", Some(matches)) => unsubscribe(config, matches),
        _ => unreachable!(),
    }
}

fn list_subs(config: &Config, args: &ArgMatches<'_>) -> CliResult {
    let game_id = value_t!(args, "game-id", u32);

    let rt = Runtime::new()?;
    let m = client(config)?;

    let filter = if let Ok(game_id) = game_id {
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

fn subscribe(config: &Config, args: &ArgMatches<'_>) -> CliResult {
    let game_id = value_t!(args, "game", u32)?;
    let mod_id = value_t!(args, "mod", u32)?;

    let rt = Runtime::new()?;
    let m = client(config)?;

    rt.block_on(m.mod_(game_id, mod_id).subscribe())?;
    Ok(())
}

fn unsubscribe(config: &Config, args: &ArgMatches<'_>) -> CliResult {
    let game_id = value_t!(args, "game", u32)?;
    let mod_id = value_t!(args, "mod", u32)?;

    let rt = Runtime::new()?;
    let m = client(config)?;

    rt.block_on(m.mod_(game_id, mod_id).unsubscribe())?;
    Ok(())
}
