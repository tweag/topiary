use crate::error::CLIResult;
use std::{
    ffi::OsString,
    io::{stdout, ErrorKind, Write},
    path::Path,
};
use tempfile::NamedTempFile;

#[derive(Debug)]
pub enum OutputFile {
    Stdout,
    Disk {
        // NOTE We stage to a file, rather than writing
        // to memory (e.g., Vec<u8>), to ensure atomicity
        staged: NamedTempFile,
        output: OsString,
    },
}

impl OutputFile {
    pub fn new(path: &str) -> CLIResult<Self> {
        match path {
            "-" => Ok(Self::Stdout),
            file => {
                // `canonicalize` if the given path exists, otherwise fallback to what was given
                let path = Path::new(file).canonicalize().or_else(|e| match e.kind() {
                    ErrorKind::NotFound => Ok(file.into()),
                    _ => Err(e),
                })?;

                // The call to `parent` will only return `None` if `path` is the root directory,
                // but that doesn't make sense as an output file, so unwrapping is safe
                let parent = path.parent().unwrap();

                Ok(Self::Disk {
                    staged: NamedTempFile::new_in(parent)?,
                    output: file.into(),
                })
            }
        }
    }

    // This function must be called to persist the output to disk
    pub fn persist(self) -> CLIResult<()> {
        if let Self::Disk { staged, output } = self {
            staged.persist(output)?;
        }

        Ok(())
    }
}

impl Write for OutputFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Stdout => stdout().write(buf),
            Self::Disk { staged, .. } => staged.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Stdout => stdout().flush(),
            Self::Disk { staged, .. } => staged.flush(),
        }
    }
}
