use clap::{ArgEnum, ArgGroup, Parser};
use std::{
    error::Error,
    ffi::OsString,
    fs::File,
    io::{stdin, stdout, BufReader, BufWriter, Write},
    path::PathBuf,
};
use tempfile::NamedTempFile;
use topiary::{formatter, FormatterResult, Language};

#[derive(ArgEnum, Clone, Copy, Debug)]
enum SupportedLanguage {
    Json,
    Toml,
    Ocaml,
    OcamlImplementation,
    OcamlInterface,
    // Any other entries in crate::Language are experimental and won't be
    // exposed in the CLI. They can be accessed using --query language/foo.scm
    // instead.
}

impl From<SupportedLanguage> for Language {
    fn from(language: SupportedLanguage) -> Self {
        match language {
            SupportedLanguage::Json => Language::Json,
            SupportedLanguage::Toml => Language::Toml,
            SupportedLanguage::Ocaml => Language::Ocaml,
            SupportedLanguage::OcamlImplementation => Language::OcamlImplementation,
            SupportedLanguage::OcamlInterface => Language::OcamlInterface,
        }
    }
}

#[derive(Debug)]
enum OutputFile {
    Stdout,
    Disk {
        // NOTE We stage to a file, rather than writing
        // to memory (e.g., Vec<u8>), to ensure atomicity
        staged: NamedTempFile,
        output: OsString,
    },
}

impl OutputFile {
    fn new(path: Option<&str>) -> FormatterResult<Self> {
        match path {
            Some("-") | None => Ok(Self::Stdout),
            Some(file) => Ok(Self::Disk {
                staged: NamedTempFile::new()?,
                output: file.into(),
            }),
        }
    }

    // This function must be called to persist the output to disk
    fn persist(self) -> FormatterResult<()> {
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

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
// Require at least one of --language, --input-file or --query (n.b., language > input > query)
#[clap(group(ArgGroup::new("rule").multiple(true).required(true).args(&["language", "input-file", "query"]),))]
struct Args {
    /// Which language to parse and format
    #[clap(short, long, arg_enum, display_order = 1)]
    language: Option<SupportedLanguage>,

    /// Path to an input file. If omitted, or equal to "-", read from standard
    /// input.
    #[clap(short = 'f', long, display_order = 2)]
    input_file: Option<String>,

    /// Which query file to use
    #[clap(short, long, display_order = 3)]
    query: Option<PathBuf>,

    /// Path to an output file. If omitted, or equal to "-", write to standard
    /// output.
    #[clap(short, long, display_order = 4)]
    output_file: Option<String>,

    /// Format the input file in place.
    #[clap(short, long, requires = "input-file", display_order = 5)]
    in_place: bool,

    /// Do not check that formatting twice gives the same output
    #[clap(short, long, display_order = 6)]
    skip_idempotence: bool,
}

fn main() {
    if let Err(e) = run() {
        print_error(&e);
        std::process::exit(1);
    }
}

fn run() -> FormatterResult<()> {
    env_logger::init();
    let args = Args::parse();

    // The as_deref() gives us an Option<&str>, which we can match against
    // string literals
    let mut input: Box<dyn std::io::Read> = match args.input_file.as_deref() {
        Some("-") | None => Box::new(stdin()),
        Some(file) => Box::new(BufReader::new(File::open(file)?)),
    };

    // NOTE If --in-place is specified, it overrides --output-file
    let mut output = BufWriter::new(if args.in_place {
        // NOTE Clap handles the case when no input file is specified. If the input file is
        // explicitly set to stdin (i.e., -), then --in-place will set the output to stdout; which
        // is not completely weird.
        OutputFile::new(args.input_file.as_deref())?
    } else {
        OutputFile::new(args.output_file.as_deref())?
    });

    let language: Option<Language> = if let Some(language) = args.language {
        Some(language.into())
    } else if let Some(filename) = args.input_file.as_deref() {
        Some(Language::detect(filename)?)
    } else {
        // At this point, Clap ensures that args.query must be present.
        // We will read the language from the query file later.
        None
    };

    let query_path = if let Some(query) = args.query {
        query
    } else if let Some(language) = language {
        // Deduce the query file from the language, if the argument is missing
        Language::query_path(language)
    } else {
        // Clap ensures we won't get here
        unreachable!();
    };

    let mut query = BufReader::new(File::open(query_path)?);

    formatter(
        &mut input,
        &mut output,
        &mut query,
        language,
        args.skip_idempotence,
    )?;

    output.into_inner()?.persist()?;

    Ok(())
}

fn print_error(e: &dyn Error) {
    eprintln!("Error: {}", e);
    if let Some(source) = e.source() {
        eprintln!("  Caused by: {}", source);
    }
}
