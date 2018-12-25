use std::fmt;
use std::io;
use std::io::prelude::*;

use md5::{self, Digest};
use tokio::prelude::*;

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

pub struct Md5 {
    digest: md5::Md5,
}

impl Md5 {
    pub fn new() -> Self {
        Self {
            digest: md5::Md5::default(),
        }
    }
}

impl Write for Md5 {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.digest.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.digest.flush()
    }
}

impl AsyncWrite for Md5 {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        Ok(().into())
    }
}

impl fmt::LowerHex for Md5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.digest.clone().result();
        fmt::LowerHex::fmt(&result, f)
    }
}

impl fmt::UpperHex for Md5 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.digest.clone().result();
        fmt::UpperHex::fmt(&result, f)
    }
}
