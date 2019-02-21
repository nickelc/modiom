use std::path::PathBuf;

use futures::{future, future::Either, Future};
use prettytable::format;
use tokio::fs::File;
use tokio::runtime::Runtime;

use modio::files::AddFileOptions;
use modiom::config::Config;
use modiom::errors::Error;
use modiom::utils::{self, Md5};

use crate::command_prelude::*;
use crate::progress::ProgressWrapper;

pub fn cli() -> App {
    subcommand("upload")
        .about("Upload new files")
        .arg(
            Arg::with_name("game")
                .help("Unique id of the game.")
                .value_name("GAME")
                .required(true)
                .validator(validate_u32),
        )
        .arg(
            Arg::with_name("mod")
                .help("Unique id of the mod.")
                .value_name("MOD")
                .required(true)
                .validator(validate_u32),
        )
        .arg(opt("filename", "Overwrite the filename.").value_name("NAME"))
        .arg(opt("version", "Version of this file release.").value_name("VERSION"))
        .arg(opt("changelog", "Changelog of this release.").value_name("CHANGELOG"))
        .arg(opt(
            "not-active",
            "When this flag is enabled, the uploaded file will not be labeled as current release.",
        ))
        .arg(opt("metadata-blob", "").value_name("BLOB"))
        .arg(opt("checksum", "Calculate the checksum before uploading."))
        .arg(
            Arg::with_name("src")
                .help("Zip file to upload.")
                .value_name("FILE")
                .required(true)
                .validator(validate_is_zip),
        )
}

pub fn exec(config: &Config, args: &ArgMatches<'_>) -> CliResult {
    let game_id = value_t!(args, "game", u32)?;
    let mod_id = value_t!(args, "mod", u32)?;
    let src = value_t!(args, "src", String).map(PathBuf::from)?;

    let mut rt = Runtime::new()?;
    let modio = config.client()?;

    let active = !args.is_present("not-active");
    let version = value_t!(args, "version", String);
    let changelog = value_t!(args, "changelog", String);
    let metadata = value_t!(args, "metadata-blob", String);

    let filename = if let Ok(filename) = value_t!(args, "filename", String) {
        filename
    } else {
        src.file_name()
            .and_then(|n| n.to_str())
            .map(|n| n.to_string())
            .ok_or_else::<Error, _>(|| "Failed to get the filename".into())?
    };

    let checksum = if args.is_present("checksum") {
        Either::A(File::open(src.clone()).and_then(|file| {
            file.metadata().and_then(|(mut file, metadata)| {
                let mut out = ProgressWrapper::new(Md5::new(), metadata.len());
                out.progress.message("calculating checksum: ");
                utils::copy(&mut file, &mut out)?;
                out.progress.finish();
                Ok(Some(format!("{:x}", out.inner())))
            })
        }))
    } else {
        Either::B(future::ok(None))
    };

    let upload = File::open(src.clone())
        .and_then(|file| file.metadata())
        .join(checksum)
        .map_err(Error::from)
        .and_then(move |((file, md), checksum)| {
            let mut file = ProgressWrapper::new(file, md.len());
            file.progress.message("uploading: ");
            let mut opts = AddFileOptions::with_read(file, filename);

            opts.active(active);

            if let Ok(version) = version {
                opts.version(version);
            }
            if let Ok(changelog) = changelog {
                opts.changelog(changelog);
            }
            if let Ok(metadata) = metadata {
                opts.metadata_blob(metadata);
            }
            if let Some(checksum) = checksum {
                opts.filehash(checksum);
            }

            modio
                .mod_(game_id, mod_id)
                .files()
                .add(opts.build())
                .map_err(Error::from)
        });

    match rt.block_on(upload) {
        Ok(file) => {
            let mut ft = table!(
                [bH2 -> "Uploaded File"],
                [b -> "Id", file.id],
                [b -> "Filename", file.filename],
                [b -> "Version", file.version.unwrap_or_else(String::new)],
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
