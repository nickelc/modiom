use prettytable::{format, Table};
use tokio::runtime::Runtime;

use modio::filter::custom_filter;
use modio::filter::prelude::*;
use modiom::config::Config;

use crate::command_prelude::*;
use crate::commands::expr;

pub fn cli() -> Command {
    Command::new("search")
        .about("Search game or mods.")
        .arg(
            opt(
                "game-id",
                "When specified, modiom will search for mods of the game.",
            )
            .value_name("ID")
            .value_parser(value_parser!(u32)),
        )
        .arg(
            opt("id", "Specify the id of the game or mod.")
                .num_args(1)
                .value_name("ID")
                .action(ArgAction::Append),
        )
        .arg(opt("name", "").value_name("VALUE"))
        .arg(opt("name-id", "").value_name("VALUE"))
        .arg(Arg::new("ft").value_name("FULLTEXT"))
        .arg(
            opt("expr", "")
                .num_args(1)
                .value_name("EXPR")
                .action(ArgAction::Append),
        )
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    let mut exprs = vec![];

    if let Some(vals) = args.get_many::<String>("expr") {
        for e in vals {
            exprs.push(expr::parse(e)?);
        }
    }
    if let Some(vals) = args.get_many::<String>("id") {
        for id in vals {
            exprs.push(expr::parse_for("id", id)?);
        }
    }
    if let Some(name) = args.get_string("name") {
        exprs.push(expr::parse_for("name", name)?);
    }
    if let Some(name_id) = args.get_string("name-id") {
        exprs.push(expr::parse_for("name_id", name_id)?);
    }
    let game_id = args.get_one::<u32>("game-id").copied();

    let mut filter = Table::new();
    filter.set_format(*format::consts::FORMAT_CLEAN);
    filter.set_titles(row![b -> "Filter"]);

    let rt = Runtime::new()?;
    let m = client(config)?;

    if let Some(game_id) = game_id {
        let mut f = Filter::default();
        for e in exprs {
            filter.add_row(row![e]);
            f = f.and(custom_filter(e.property, e.op.into(), e.right.into_value()));
        }
        if let Some(ft) = args.get_string("ft") {
            filter.add_row(row![format!("fulltext = {:?}", ft)]);
            f = f.and(Fulltext::eq(ft));
        }
        if !filter.is_empty() {
            filter.printstd();
            println!();
        }

        let list = rt.block_on(m.game(game_id).mods().search(f).first_page());
        if let Ok(list) = list {
            let mut output = Table::new();
            output.set_format(*format::consts::FORMAT_CLEAN);
            output.add_row(row!(
                b -> "Id",
                b -> "Name-Id",
                b -> "Name",
            ));
            for m in list {
                output.add_row(row![m.id, m.name_id, m.name]);
            }
            if output.is_empty() {
                output.add_row(row![H3 -> "No results"]);
            }
            output.printstd();
        }
    } else {
        let mut f = Filter::default();
        for e in exprs {
            filter.add_row(row![e]);
            f = f.and(custom_filter(e.property, e.op.into(), e.right.into_value()));
        }
        if let Some(ft) = args.get_string("ft") {
            filter.add_row(row![format!("fulltext = {:?}", ft)]);
            f = f.and(Fulltext::eq(ft));
        }
        if !filter.is_empty() {
            filter.printstd();
            println!();
        }

        let list = rt.block_on(m.games().search(f).first_page());
        if let Ok(list) = list {
            let mut output = Table::new();
            output.set_format(*format::consts::FORMAT_CLEAN);
            output.set_titles(row![
                b -> "Id",
                b -> "Name-Id",
                b -> "Name",
            ]);
            for g in list {
                output.add_row(row![g.id, g.name_id, g.name]);
            }
            if output.is_empty() {
                output.add_row(row![H3 -> "No results"]);
            }
            output.printstd();
        }
    }
    Ok(())
}
