use crate::error::CLIResult;
use std::{
    ffi::OsString,
    fs::File,
    io::{stdin, stdout, ErrorKind, Read, Result, Write},
    path::{Path, PathBuf},
};
use tempfile::NamedTempFile;

enum InputSource {
    Stdin,
    Disk(File),
}

pub struct InputFile {
    source: InputSource,
}

impl InputFile {
    pub fn new(path: &str) -> CLIResult<Self> {
        Ok(match path {
            "-" => InputFile {
                source: InputSource::Stdin,
            },
            file => InputFile {
                source: InputSource::Disk(File::open(file)?),
            },
        })
    }
}

impl Read for InputFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match self.source {
            InputSource::Stdin => stdin().lock().read(buf),
            InputSource::Disk(ref mut file) => file.read(buf),
        }
    }
}

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
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self {
            Self::Stdout => stdout().lock().write(buf),
            Self::Disk { staged, .. } => staged.write(buf),
        }
    }

    fn flush(&mut self) -> Result<()> {
        match self {
            Self::Stdout => stdout().lock().flush(),
            Self::Disk { staged, .. } => staged.flush(),
        }
    }
}
