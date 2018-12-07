use std::env;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};

pub struct TempFile {
    inner: Option<File>,
    path: PathBuf,
}

impl TempFile {
    pub fn create<P: AsRef<Path>>(file_path: P) -> Result<TempFile> {
        let mut path = env::temp_dir();
        path.push(file_path);
        let inner = Some(File::create(path.clone())?);

        Ok(TempFile { inner, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn close(&mut self) {
        self.inner = None;
    }

    pub fn reopen(&mut self) -> Result<()> {
        if self.inner.is_some() {
            return Err(Error::new("Already opened"));
        }

        self.inner = Some(File::open(self.path.clone())?);
        Ok(())
    }
}

impl Read for TempFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self.inner {
            Some(ref mut f) => f.read(buf),
            None => Err(io::Error::from(io::ErrorKind::BrokenPipe)),
        }
    }
}

impl Write for TempFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner {
            Some(ref mut f) => f.write(buf),
            None => Err(io::Error::from(io::ErrorKind::BrokenPipe)),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self.inner {
            Some(ref mut f) => f.flush(),
            None => Err(io::Error::from(io::ErrorKind::BrokenPipe)),
        }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}
