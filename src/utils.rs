use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

pub fn find_manifest_for_wd(cwd: &Path) -> io::Result<PathBuf> {
    let file = "Modio.toml";
    cwd.ancestors()
        .map(|p| p.join(file))
        .find(|p| fs::metadata(p).is_ok())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "Could not find `{}` in `{}` or any parent directory",
                    file,
                    cwd.display()
                ),
            )
        })
}

pub fn read(path: &Path) -> io::Result<String> {
    let mut ret = String::new();
    let mut f = fs::File::open(path)?;
    f.read_to_string(&mut ret)?;
    Ok(ret)
}

pub fn copy<R: ?Sized, W: ?Sized>(reader: &mut R, writer: &mut W) -> io::Result<u64>
where
    R: Read,
    W: Write,
{
    let mut buf = vec![0; 512 * 512];
    let mut written = 0;
    loop {
        let len = match reader.read(&mut buf) {
            Ok(0) => return Ok(written),
            Ok(len) => len,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => continue,
            Err(e) => return Err(e),
        };
        writer.write_all(&buf[..len])?;
        written += len as u64;
    }
}
