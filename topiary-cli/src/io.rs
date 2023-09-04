use std::{
    ffi::OsString,
    fmt,
    fs::File,
    io::{stdin, stdout, ErrorKind, Read, Result, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;
use topiary::{Configuration, Language, SupportedLanguage, TopiaryQuery};

use crate::{
    cli::{AtLeastOneInput, ExactlyOneInput, FromStdin},
    error::{CLIResult, TopiaryError},
    language::LanguageDefinition,
};

type QuerySource = PathBuf;

/// Unified interface for input sources. We either have input from:
/// * Standard input, in which case we need to specify the language and, optionally, query override
/// * A sequence of files
///
/// These are captured by the CLI parser, with `cli::AtLeastOneInput` and `cli::ExactlyOneInput`.
/// We use this struct to normalise the interface for downstream (using `From` implementations).
pub enum InputFrom {
    Stdin(SupportedLanguage, Option<QuerySource>),
    Files(Vec<PathBuf>),
}

impl From<&ExactlyOneInput> for InputFrom {
    fn from(input: &ExactlyOneInput) -> Self {
        match input {
            ExactlyOneInput {
                stdin: Some(FromStdin { language, query }),
                ..
            } => InputFrom::Stdin(language.to_owned(), query.to_owned()),

            ExactlyOneInput {
                file: Some(path), ..
            } => InputFrom::Files(vec![path.to_owned()]),

            _ => unreachable!("Clap guarantees input is always one of the above"),
        }
    }
}

impl From<&AtLeastOneInput> for InputFrom {
    fn from(input: &AtLeastOneInput) -> Self {
        match input {
            AtLeastOneInput {
                stdin: Some(FromStdin { language, query }),
                ..
            } => InputFrom::Stdin(language.to_owned(), query.to_owned()),

            AtLeastOneInput { files, .. } => InputFrom::Files(files.to_owned()),
        }
    }
}

/// Each `InputFile` needs to locate its source (standard input or disk), such that its `io::Read`
/// implementation can do the right thing.
#[derive(Debug)]
pub enum InputSource {
    Stdin,
    Disk(PathBuf, Option<File>),
}

impl fmt::Display for InputSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdin => write!(f, "standard input"),
            Self::Disk(path, _) => write!(f, "{}", path.to_string_lossy()),
        }
    }
}

/// An `InputFile` is the unit of input for Topiary, encapsulating everything needed for downstream
/// processing. It implements `io::Read`, so it can be passed directly to the Topiary API.
#[derive(Debug)]
pub struct InputFile<'cfg> {
    source: InputSource,
    language: &'cfg Language,
    query: QuerySource,
}

impl<'cfg> InputFile<'cfg> {
    /// Convert our `InputFile` into language definition values that Topiary can consume
    pub async fn to_language_definition(&self) -> CLIResult<LanguageDefinition> {
        let grammar = self.language.grammar().await?;
        let query = {
            let contents = tokio::fs::read_to_string(&self.query).await?;
            TopiaryQuery::new(&grammar, &contents)?
        };

        Ok(LanguageDefinition {
            query,
            language: self.language.clone(),
            grammar,
        })
    }

    /// Expose input source
    pub fn source(&self) -> &InputSource {
        &self.source
    }

    /// Expose language for input
    pub fn language(&self) -> &Language {
        self.language
    }

    /// Expose query path for input
    pub fn query(&self) -> &PathBuf {
        &self.query
    }
}

impl<'cfg> Read for InputFile<'cfg> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        match &mut self.source {
            InputSource::Stdin => stdin().lock().read(buf),

            InputSource::Disk(path, fd) => {
                if fd.is_none() {
                    *fd = Some(File::open(path)?);
                }

                fd.as_mut().unwrap().read(buf)
            }
        }
    }
}

/// `Inputs` is an iterator of fully qualified `InputFile`s, each wrapped in `CLIResult`, which is
/// populated by its constructor from any type that implements `Into<InputFrom>`
pub struct Inputs<'cfg>(Vec<CLIResult<InputFile<'cfg>>>);

impl<'cfg, 'i> Inputs<'cfg> {
    pub fn new<T>(config: &'cfg Configuration, inputs: &'i T) -> Self
    where
        &'i T: Into<InputFrom>,
    {
        let inputs = match inputs.into() {
            InputFrom::Stdin(language, query) => {
                vec![(|| {
                    let language = language.to_language(config);
                    let query = query.unwrap_or(language.query_file()?);

                    Ok(InputFile {
                        source: InputSource::Stdin,
                        language,
                        query,
                    })
                })()]
            }

            InputFrom::Files(files) => files
                .into_iter()
                .map(|path| {
                    let language = Language::detect(&path, config)?;
                    let query = language.query_file()?;

                    Ok(InputFile {
                        source: InputSource::Disk(path, None),
                        language,
                        query,
                    })
                })
                .collect(),
        };

        Self(inputs)
    }
}

impl<'cfg> Iterator for Inputs<'cfg> {
    type Item = CLIResult<InputFile<'cfg>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

/// An `OutputFile` is the unit of output for Topiary, differentiating between standard output and
/// disk (which uses temporary files to perform atomic updates in place). It implements
/// `io::Write`, so it can be passed directly to the Topiary API.
///
/// NOTE When writing to disk, the `persist` function must be called to perform the in place write.
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

impl fmt::Display for OutputFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdout => write!(f, "standard ouput"),
            Self::Disk { output, .. } => write!(f, "{}", output.to_string_lossy()),
        }
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

// Convenience conversion:
// * stdin maps to stdout
// * Files map to themselves (i.e., for in-place updates)
impl<'cfg> TryFrom<&InputFile<'cfg>> for OutputFile {
    type Error = TopiaryError;

    fn try_from(input: &InputFile) -> CLIResult<Self> {
        match &input.source {
            InputSource::Stdin => Ok(Self::Stdout),
            InputSource::Disk(path, _) => Self::new(path.to_string_lossy().as_ref()),
        }
    }
}
