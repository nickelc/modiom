use std::io;
use std::io::prelude::*;

use pbr::{ProgressBar, Units};
use tokio::prelude::*;

pub struct ProgressWrapper<T, W>
where
    W: Write,
{
    inner: T,
    pub progress: ProgressBar<W>,
}

impl<T> ProgressWrapper<T, io::Stdout> {
    pub fn new(inner: T, total: u64) -> Self {
        let mut progress = ProgressBar::new(total);
        progress.set_units(Units::Bytes);

        ProgressWrapper { inner, progress }
    }
}

impl<T, W: Write> ProgressWrapper<T, W> {
    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn finish(&mut self) {
        self.progress.finish();
    }
}

impl<T: Read, W: Write> AsyncRead for ProgressWrapper<T, W> {}

impl<T: AsyncWrite, W: Write> AsyncWrite for ProgressWrapper<T, W> {
    fn shutdown(&mut self) -> Poll<(), io::Error> {
        self.inner.shutdown()
    }
}

impl<T: Read, W: Write> Read for ProgressWrapper<T, W> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let ret = self.inner.read(buf);
        if let Ok(n) = ret {
            self.progress.add(n as u64);
        }
        ret
    }
}

impl<T: Write, W: Write> Write for ProgressWrapper<T, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let ret = self.inner.write(buf);
        if let Ok(n) = ret {
            self.progress.add(n as u64);
        }
        ret
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
