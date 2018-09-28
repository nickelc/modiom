use tokio::runtime::Runtime;

use command_prelude::*;
use commands::expr;

use modio::auth::Credentials;
use modio::games::GamesListOptions;
use modio::mods::ModsListOptions;
use modio::Modio;
use modiom::config::Config;

pub fn cli() -> App {
    subcommand("search")
        .arg(opt("game-id", "").value_name("ID").validator(validate_u32))
        .arg(
            opt("id", "")
                .multiple(true)
                .number_of_values(1)
                .value_name("ID"),
        ).arg(opt("name", "").value_name("VALUE"))
        .arg(opt("name-id", "").value_name("VALUE"))
        .arg(Arg::with_name("ft").value_name("FULLTEXT"))
        .arg(
            opt("expr", "")
                .multiple(true)
                .number_of_values(1)
                .value_name("EXPR"),
        )
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    let mut exprs = vec![];

    if let Some(vals) = args.values_of("expr") {
        for e in vals {
            exprs.push(expr::parse(e)?);
        }
    }
    if let Some(vals) = args.values_of("id") {
        for id in vals {
            exprs.push(expr::parse_for("id", id)?);
        }
    }
    if let Some(name) = args.value_of("name") {
        exprs.push(expr::parse_for("name", name)?);
    }
    if let Some(name_id) = args.value_of("name-id") {
        exprs.push(expr::parse_for("name_id", name_id)?);
    }
    let game_id = value_t!(args, "game-id", u32);

    if let Ok(Some(token)) = config.auth_token() {
        let mut rt = Runtime::new()?;
        let m = Modio::host(
            config.host(),
            "modiom",
            Credentials::Token(token),
        );

        if let Ok(game_id) = game_id {
            let mut opts = ModsListOptions::new();
            for e in exprs {
                opts.add_filter(e.property, e.op.into(), e.right.to_value());
            }
            if let Some(ft) = args.value_of("ft") {
                opts.fulltext(ft);
            }
            println!("{:#?}", opts);
            let list = rt.block_on(m.game(game_id).mods().list(&opts));
            if let Ok(list) = list {
                for m in list {
                    println!("{:?}\tid={}", m.name, m.id);
                }
            }
        } else {
            let mut opts = GamesListOptions::new();
            for e in exprs {
                opts.add_filter(e.property, e.op.into(), e.right.to_value());
            }
            if let Some(ft) = args.value_of("ft") {
                opts.fulltext(ft);
            }
            println!("{:#?}", opts);
            let list = rt.block_on(m.games().list(&opts));
            if let Ok(list) = list {
                for g in list {
                    println!("{:?}\tid={}", g.name, g.id);
                }
            }
        }
    }
    Ok(())
}
