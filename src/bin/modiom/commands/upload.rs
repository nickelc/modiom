use std::borrow::Cow;
use std::path::PathBuf;

use futures::{future::try_join3, StreamExt};
use prettytable::format;
use tokio::fs::{self, File};
use tokio::io::BufReader;
use tokio::runtime::Runtime;
use tokio_util::io::ReaderStream;

use modio::files::AddFileOptions;
use modio::types::id::{GameId, ModId};
use modiom::config::Config;
use modiom::md5::Md5;

use crate::command_prelude::*;

pub fn cli() -> Command {
    Command::new("upload")
        .about("Upload new files")
        .arg(
            Arg::new("game")
                .help("Unique id of the game.")
                .value_name("GAME")
                .required(true)
                .value_parser(value_parser!(GameId)),
        )
        .arg(
            Arg::new("mod")
                .help("Unique id of the mod.")
                .value_name("MOD")
                .required(true)
                .value_parser(value_parser!(ModId)),
        )
        .arg(opt("filename", "Overwrite the filename.").value_name("NAME"))
        .arg(opt("version", "Version of this file release.").value_name("VERSION"))
        .arg(opt("changelog", "Changelog of this release.").value_name("CHANGELOG"))
        .arg(
            opt(
                "not-active",
                "The uploaded file will not be labeled as current release.",
            )
            .action(ArgAction::SetTrue),
        )
        .arg(opt("metadata-blob", "").value_name("BLOB"))
        .arg(opt("checksum", "Calculate the checksum before uploading.").action(ArgAction::SetTrue))
        .arg(
            Arg::new("src")
                .help("Zip file to upload.")
                .value_name("FILE")
                .required(true)
                .value_parser(PathBufValueParser::new().try_map(|path| {
                    if path.is_file() && path.ends_with(".zip") {
                        return Ok(path);
                    }
                    Err(String::from("File is not a zip."))
                })),
        )
}

pub fn exec(config: &Config, args: &ArgMatches) -> CliResult {
    let game_id = *args.get_one("game").expect("required arg");
    let mod_id = *args.get_one("mod").expect("required arg");
    let src = args
        .get_string("src")
        .map(PathBuf::from)
        .expect("required arg");

    let rt = Runtime::new()?;
    let modio = client(config)?;

    let active = !args.get_flag("not-active");
    let version = args.get_string("version");
    let changelog = args.get_string("changelog");
    let metadata = args.get_string("metadata-blob");

    let filename: Cow<_> = if let Some(filename) = args.get_string("filename") {
        filename.into()
    } else {
        src.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_string())
            .ok_or("Failed to get the filename")?
            .into()
    };

    let checksum = async {
        if args.get_flag("checksum") {
            let r = File::open(&src).await?;
            let r = BufReader::with_capacity(512 * 512, r);
            let st = ReaderStream::new(r);
            let mut md5 = Md5::default();
            st.forward(&mut md5).await?;

            Ok(Some(md5.into_lower_hex()))
        } else {
            Ok(None)
        }
    };

    let upload = async {
        let file = File::open(&src);
        let md = fs::metadata(&src);

        let (file, _md, checksum) = try_join3(file, md, checksum).await?;
        let mut opts = AddFileOptions::with_read(file, filename);

        opts = opts.active(active);

        if let Some(version) = version {
            opts = opts.version(version);
        }
        if let Some(changelog) = changelog {
            opts = opts.changelog(changelog);
        }
        if let Some(metadata) = metadata {
            opts = opts.metadata_blob(metadata);
        }
        if let Some(checksum) = checksum {
            opts = opts.filehash(checksum);
        }

        let file = modio.mod_(game_id, mod_id).files().add(opts).await?;

        Ok::<_, Box<dyn std::error::Error>>(file)
    };

    match rt.block_on(upload) {
        Ok(file) => {
            let mut ft = table!(
                [bH2 -> "Uploaded File"],
                [b -> "Id", file.id],
                [b -> "Filename", file.filename],
                [b -> "Version", file.version.unwrap_or_default()],
                [b -> "Download", file.download.binary_url],
                [b -> "Size", file.filesize],
                [b -> "MD5", file.filehash.md5]
            );
            ft.set_format(*format::consts::FORMAT_CLEAN);
            ft.printstd();
        }
        Err(e) => println!("{}", e),
    }
    Ok(())
}
