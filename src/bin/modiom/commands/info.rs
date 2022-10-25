use futures::{future, TryFutureExt};
use prettytable::format;
use textwrap::fill;
use tokio::runtime::Runtime;

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

    let rt = Runtime::new()?;
    let modio = client(config)?;

    let modref = modio.mod_(game_id, mod_id);

    let files = async {
        if args.is_present("files") {
            let f = Default::default();
            modref.files().search(f).first_page().map_ok(Some).await
        } else {
            Ok(None)
        }
    };

    let modref = modio.mod_(game_id, mod_id);

    let stats = async {
        if args.is_present("stats") {
            modref.statistics().map_ok(Some).await
        } else {
            Ok(None)
        }
    };

    let modref = modio.mod_(game_id, mod_id);
    let deps = modref.dependencies().list();
    let mod_ = modref.get();
    let task = future::try_join4(mod_, deps, stats, files);

    match rt.block_on(task) {
        Ok((m, deps, stats, files)) => {
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
                [b -> "Homepage", m.homepage_url.map(|u| u.to_string()).unwrap_or_default()],
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
                mt.add_row(row![b -> "Version", file.version.unwrap_or_default()]);
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
                        file.version.unwrap_or_default(),
                        file.download.binary_url
                    ]);
                }
                ft.printstd();
            }
        }
        Err(e) => println!("{}", e),
    };
    Ok(())
}
