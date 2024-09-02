use std::{
    ffi::OsString,
    fmt::{self, Display},
    fs::File,
    io::{stdin, stdout, ErrorKind, Read, Result, Write},
    path::{Path, PathBuf},
};

use tempfile::NamedTempFile;
use topiary_config::Configuration;
use topiary_core::{Language, TopiaryQueries};

use crate::{
    cli::{AtLeastOneInput, ExactlyOneInput, FromStdin},
    error::{CLIError, CLIResult, TopiaryError},
};

#[derive(Debug, Clone, Hash)]
pub enum QuerySource {
    Path(PathBuf),
    BuiltIn(String),
}

impl From<PathBuf> for QuerySource {
    fn from(path: PathBuf) -> Self {
        QuerySource::Path(path)
    }
}

impl From<&PathBuf> for QuerySource {
    fn from(path: &PathBuf) -> Self {
        QuerySource::Path(path.clone())
    }
}

impl From<&str> for QuerySource {
    fn from(string: &str) -> Self {
        QuerySource::BuiltIn(String::from(string))
    }
}

impl Display for QuerySource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QuerySource::Path(p) => write!(f, "{}", p.to_string_lossy()),
            QuerySource::BuiltIn(_) => write!(f, "built-in query"),
        }
    }
}

/// Unified interface for input sources. We either have input from:
/// * Standard input, in which case we need to specify the language and, optionally, query override
/// * A sequence of files
///
/// These are captured by the CLI parser, with `cli::AtLeastOneInput` and `cli::ExactlyOneInput`.
/// We use this struct to normalise the interface for downstream (using `From` implementations).
pub enum InputFrom {
    Stdin(String, Option<QuerySource>),
    Files(Vec<PathBuf>),
}

impl From<&ExactlyOneInput> for InputFrom {
    fn from(input: &ExactlyOneInput) -> Self {
        match input {
            ExactlyOneInput {
                stdin: Some(FromStdin { language, query }),
                ..
            } => InputFrom::Stdin(language.to_owned(), query.as_ref().map(|p| p.into())),

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
            } => InputFrom::Stdin(language.to_owned(), query.as_ref().map(|p| p.into())),

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
    language: &'cfg topiary_config::language::Language,
    formatting_query: QuerySource,
    injection_query: Option<QuerySource>,
}

impl<'cfg> InputFile<'cfg> {
    /// Convert our `InputFile` into language definition values that Topiary can consume
    pub async fn to_language(&self) -> CLIResult<Language> {
        let grammar = self.language().grammar()?;
        let contents = match &self.formatting_query {
            QuerySource::Path(query) => tokio::fs::read_to_string(query).await?,
            QuerySource::BuiltIn(contents) => contents.to_owned(),
        };
        let query = TopiaryQueries::new(&grammar, &contents, None)?;

        Ok(Language {
            name: self.language.name.clone(),
            query,
            grammar,
            indent: self.language().config.indent.clone(),
        })
    }

    /// Expose input source
    pub fn source(&self) -> &InputSource {
        &self.source
    }

    /// Expose language for input
    pub fn language(&self) -> &topiary_config::language::Language {
        self.language
    }

    /// Expose query path for input
    pub fn formatting_query(&self) -> &QuerySource {
        &self.formatting_query
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
            InputFrom::Stdin(language_name, query) => {
                vec![(|| {
                    let language = config.get_language(&language_name)?;
                    let (formatting_query, injection_query): (QuerySource, Option<QuerySource>) =
                        match query {
                            // The user specified a query file
                            Some(p) => (p, None),
                            // The user did not specify a file, try the default locations
                            None => match language.find_query_files() {
                                Ok((formatting_query, injection_query)) => {
                                    (formatting_query.into(), injection_query.map(Into::into))
                                }
                                // For some reason, Topiary could not find any
                                // matching file in a default location. As a final attempt, use try to the the
                                // builtin ones. Store the error, return that if we
                                // fail to find anything, because the builtin error might be unexpected.
                                Err(e) => {
                                    log::warn!("No query files found in any of the expected locations. Falling back to compile-time included files.");
                                    to_queries(&language_name).map_err(|_| e)?
                                }
                            },
                        };

                    Ok(InputFile {
                        source: InputSource::Stdin,
                        language,
                        formatting_query,
                        injection_query,
                    })
                })()]
            }

            InputFrom::Files(files) => files
                .into_iter()
                .map(|path| {
                    let language = config.detect(&path)?;
                    let (formatting_query, injection_query) = language.find_query_files()?;

                    Ok(InputFile {
                        source: InputSource::Disk(path, None),
                        language,
                        formatting_query: formatting_query.into(),
                        injection_query: injection_query.map(Into::into),
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

fn to_queries<T>(name: T) -> CLIResult<(QuerySource, Option<QuerySource>)>
where
    T: AsRef<str> + fmt::Display,
{
    let (formatting_query, injection_query) = match name.as_ref() {
        #[cfg(feature = "bash")]
        "bash" => Ok(topiary_queries::bash()),

        #[cfg(feature = "css")]
        "css" => Ok(topiary_queries::css()),

        #[cfg(feature = "json")]
        "json" => Ok(topiary_queries::json()),

        #[cfg(feature = "nickel")]
        "nickel" => Ok(topiary_queries::nickel()),

        #[cfg(feature = "ocaml")]
        "ocaml" => Ok(topiary_queries::ocaml()),

        #[cfg(feature = "ocaml_interface")]
        "ocaml_interface" => Ok(topiary_queries::ocaml_interface()),

        #[cfg(feature = "ocamllex")]
        "ocamllex" => Ok(topiary_queries::ocamllex()),

        #[cfg(feature = "rust")]
        "rust" => Ok(topiary_queries::rust()),

        #[cfg(feature = "toml")]
        "toml" => Ok(topiary_queries::toml()),

        #[cfg(feature = "tree_sitter_query")]
        "tree_sitter_query" => Ok(topiary_queries::tree_sitter_query()),

        name => Err(TopiaryError::Bin(
            format!("The specified language is unsupported: {}", name),
            Some(CLIError::UnsupportedLanguage(name.to_string())),
        )),
    }?;

    let formatting_query = formatting_query.into();

    let injection_query = injection_query.map(Into::into);

    Ok((formatting_query, injection_query))
}
