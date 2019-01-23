use futures::{future, future::Either, Future};
use prettytable::format;
use textwrap::fill;
use tokio::runtime::Runtime;

use modio::auth::Credentials;
use modio::Modio;
use modiom::config::Config;

use crate::command_prelude::*;

pub fn cli() -> App {
    subcommand("info")
        .about("Show information of mods")
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
        )
        .arg(opt("files", "List all files."))
        .arg(opt("stats", "Show the statistics."))
}

pub fn exec(config: &Config, args: &ArgMatches<'_>) -> CliResult {
    let game_id = value_t!(args, "game", u32)?;
    let mod_id = value_t!(args, "mod", u32)?;

    if let Ok(Some(token)) = config.auth_token() {
        let mut rt = Runtime::new()?;
        let modio = Modio::host(config.host(), "modiom", Credentials::Token(token));

        let modref = modio.mod_(game_id, mod_id);

        let files = if args.is_present("files") {
            Either::A(modref.files().list(&Default::default()).map(Some))
        } else {
            Either::B(future::ok(None))
        };

        let stats = if args.is_present("stats") {
            Either::A(modref.statistics().map(Some))
        } else {
            Either::B(future::ok(None))
        };

        let mod_ = modref.get();
        let deps = modref.dependencies().list();
        let task = mod_.join(deps).join(stats.join(files));

        match rt.block_on(task) {
            Ok(((m, deps), (stats, files))) => {
                let tags = m
                    .tags
                    .iter()
                    .map(|t| format!("{:?}", t.name))
                    .collect::<Vec<_>>()
                    .join(", ");
                let deps = deps.into_iter().map(|d| d.mod_id).collect::<Vec<_>>();

                let mut mt = table!(
                    [b -> "Id", m.id],
                    [b -> "Name-Id", m.name_id],
                    [b -> "Name", m.name],
                    [b -> "Summary", fill(&m.summary, 60)],
                    [b -> "Profile", m.profile_url],
                    [b -> "Homepage", m.homepage_url.map(|u| u.to_string()).unwrap_or_else(String::new)],
                    [b -> "Tags", format!("[{}]", tags)],
                    [b -> "Dependencies", format!("{:?}", deps)]
                );
                let mut primary = 0;
                mt.set_format(*format::consts::FORMAT_CLEAN);
                if let Some(file) = m.modfile {
                    primary = file.id;
                    mt.add_empty_row();
                    mt.add_row(row![bH2 -> "File"]);
                    mt.add_row(row![b -> "Id", file.id]);
                    mt.add_row(row![b -> "Filename", file.filename]);
                    mt.add_row(row![b -> "Version", file.version.unwrap_or_else(String::new)]);
                    mt.add_row(row![b -> "Download", file.download.binary_url]);
                    mt.add_row(row![b -> "Size", file.filesize]);
                    mt.add_row(row![b -> "MD5", file.filehash.md5]);
                }
                if let Some(stats) = stats {
                    mt.add_empty_row();
                    mt.add_row(row![bH2 -> "Statistics"]);
                    mt.add_row(row![b -> "Downloads", stats.downloads_total]);
                    mt.add_row(row![b -> "Subscribers", stats.subscribers_total]);
                    mt.add_row(row![
                        b -> "Rank",
                        format!(
                            "{}/{}",
                            stats.popularity.rank_position,
                            stats.popularity.rank_total,
                        )
                    ]);
                    mt.add_row(row![
                        b -> "Ratings",
                        format!(
                            "{} / total: {} positive: {} negative: {}",
                            stats.ratings.display_text,
                            stats.ratings.total,
                            stats.ratings.positive,
                            stats.ratings.negative,
                        )
                    ]);
                }
                mt.printstd();

                if let Some(files) = files {
                    let mut ft = table!(
                        [],
                        [bH4 -> "Files"],
                        [b -> "Id", b -> "Filename", b -> "Version", b -> "Download"]
                    );
                    ft.set_format(*format::consts::FORMAT_CLEAN);
                    for file in files {
                        let suffix = if primary == file.id { "*" } else { "" };
                        ft.add_row(row![
                            format!("{}{}", file.id, suffix),
                            file.filename,
                            file.version.unwrap_or_else(String::new),
                            file.download.binary_url
                        ]);
                    }
                    ft.printstd();
                }
            }
            Err(e) => println!("{}", e),
        };
    }
    Ok(())
}
