//! Topiary can be configured using the `Configuration` struct.
//! A basic configuration, written in Nickel, is included at build time and parsed at runtime.
//! Additional configuration has to be provided by the user of the library.
pub mod error;
pub mod language;
pub mod source;

use std::{
    collections::HashMap,
    fmt,
    path::{Path, PathBuf},
};

use language::{Language, LanguageConfiguration};
use nickel_lang_core::{
    error::NullReporter, eval::cache::CacheImpl, program::Program, term::RichTerm,
};
use serde::Deserialize;

#[cfg(not(target_arch = "wasm32"))]
use crate::error::TopiaryConfigFetchingError;
#[cfg(not(target_arch = "wasm32"))]
use tempfile::tempdir;

use crate::error::{TopiaryConfigError, TopiaryConfigResult};

pub use source::Source;

/// The configuration of the Topiary.
///
/// Contains information on how to format every language the user is interested in, modulo what is
/// supported. It can be provided by the user of the library, or alternatively, Topiary ships with
/// default configuration that can be accessed using `Configuration::default`.
#[derive(Debug)]
pub struct Configuration {
    languages: Vec<Language>,
}

/// Internal struct to help with deserialisation, converted to the actual Configuration in deserialization
#[derive(Debug, serde::Deserialize, PartialEq, serde::Serialize, Clone)]
struct SerdeConfiguration {
    languages: HashMap<String, LanguageConfiguration>,
}

impl Configuration {
    /// Consume the configuration from the usual sources.
    /// Which sources exactly can be read in the documentation of `Source`.
    ///
    /// # Errors
    ///
    /// If the configuration file does not exist, this function will return a `TopiaryConfigError`
    /// with the path that was not found.
    /// If the configuration file exists, but cannot be parsed, this function will return a
    /// `TopiaryConfigError` with the error that occurred.
    #[allow(clippy::result_large_err)]
    pub fn fetch(merge: bool, file: &Option<PathBuf>) -> TopiaryConfigResult<(Self, RichTerm)> {
        // If we have an explicit file, fail if it doesn't exist
        if let Some(path) = file
            && !path.exists()
        {
            return Err(TopiaryConfigError::FileNotFound(path.to_path_buf()));
        }

        if merge {
            // Get all available configuration sources
            let sources: Vec<Source> = Source::fetch_all(file);

            // And ask Nickel to parse and merge them
            Self::parse_and_merge(&sources)
        } else {
            // Get the available configuration with best priority
            match Source::fetch_one(file) {
                Source::Builtin => Self::parse(Source::Builtin),
                source => Self::parse_and_merge(&[source, Source::Builtin]),
            }
        }
    }

    /// Gets a language configuration from the entire configuration.
    ///
    /// # Errors
    ///
    /// If the provided language name cannot be found in the `Configuration`, this
    /// function returns a `TopiaryConfigError`
    #[allow(clippy::result_large_err)]
    pub fn get_language<T>(&self, name: T) -> TopiaryConfigResult<&Language>
    where
        T: AsRef<str> + fmt::Display,
    {
        self.languages
            .iter()
            .find(|language| language.name == name.as_ref())
            .ok_or(TopiaryConfigError::UnknownLanguage(name.to_string()))
    }

    /// Prefetch a language per its configuration
    ///
    /// # Errors
    ///
    /// If any grammar could not build, a `TopiaryConfigFetchingError` is returned.
    #[cfg(not(target_arch = "wasm32"))]
    fn fetch_language(
        language: &Language,
        force: bool,
        tmp_dir: &Path,
    ) -> Result<(), TopiaryConfigFetchingError> {
        match &language.config.grammar.source {
            language::GrammarSource::Git(git_source) => {
                let library_path = language.library_path()?;

                log::info!(
                    "Fetch \"{}\": Configured via Git ({} ({})); to {}",
                    language.name,
                    git_source.git,
                    git_source.rev,
                    library_path.display()
                );

                git_source.fetch_and_compile_with_dir(
                    &language.name,
                    library_path,
                    force,
                    tmp_dir.to_path_buf(),
                )
            }

            language::GrammarSource::Path(path) => {
                log::info!(
                    "Fetch \"{}\": Configured via filesystem ({}); nothing to do",
                    language.name,
                    path.display(),
                );

                if !path.exists() {
                    Err(TopiaryConfigFetchingError::GrammarFileNotFound(
                        path.to_path_buf(),
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Prefetches and builds the desired language.
    /// This can be beneficial to speed up future startup time.
    ///
    /// # Errors
    ///
    /// If the language could not be found or the Grammar could not be build, a `TopiaryConfigError` is returned.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::result_large_err)]
    pub fn prefetch_language<T>(&self, language: T, force: bool) -> TopiaryConfigResult<()>
    where
        T: AsRef<str> + fmt::Display,
    {
        let tmp_dir = tempdir()?;
        let tmp_dir_path = tmp_dir.path().to_owned();
        let l = self.get_language(language)?;
        Configuration::fetch_language(l, force, &tmp_dir_path)?;
        Ok(())
    }

    /// Prefetches and builds all known languages.
    /// This can be beneficial to speed up future startup time.
    ///
    /// # Errors
    ///
    /// If any Grammar could not be build, a `TopiaryConfigError` is returned.
    #[cfg(not(target_arch = "wasm32"))]
    #[allow(clippy::result_large_err)]
    pub fn prefetch_languages(&self, force: bool) -> TopiaryConfigResult<()> {
        let tmp_dir = tempdir()?;
        let tmp_dir_path = tmp_dir.path().to_owned();

        // When the `parallel` feature is enabled (which it is by default), we use Rayon to fetch
        // and compile all found grammars concurrently.
        // NOTE The MSVC linker does not seem to like concurrent builds, so concurrency is disabled
        // on Windows (see https://github.com/tweag/topiary/issues/868)
        #[cfg(all(feature = "parallel", not(windows)))]
        {
            use rayon::prelude::*;
            self.languages
                .par_iter()
                .map(|l| Configuration::fetch_language(l, force, &tmp_dir_path))
                .collect::<Result<Vec<_>, TopiaryConfigFetchingError>>()?;
        }

        #[cfg(any(not(feature = "parallel"), windows))]
        {
            self.languages
                .iter()
                .map(|l| Configuration::fetch_language(l, force, &tmp_dir_path))
                .collect::<Result<Vec<_>, TopiaryConfigFetchingError>>()?;
        }

        tmp_dir.close()?;
        Ok(())
    }

    /// Detects the language from a file path.
    ///
    /// First attempts to match by file extension. If multiple languages share the same
    /// extension, uses the tree-sitter-shebang grammar to parse the first line and
    /// disambiguate based on the interpreter.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file being analyzed
    ///
    /// # Returns
    ///
    /// * `Ok(&Language)` if a language was successfully detected
    /// * `Err(TopiaryConfigError)` if detection failed
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The file has no extension
    /// - The extension is not associated with any configured language
    #[allow(clippy::result_large_err)]
    pub fn detect<P: AsRef<Path>>(&self, path: P) -> TopiaryConfigResult<&Language> {
        let pb = &path.as_ref().to_path_buf();

        if let Some(extension) = pb.extension().and_then(|ext| ext.to_str()) {
            // Collect all languages matching this extension
            let matching_langs: Vec<&Language> = self
                .languages
                .iter()
                .filter(|lang| lang.config.extensions.contains(extension))
                .collect();

            match matching_langs.len() {
                0 => return Err(TopiaryConfigError::UnknownExtension(extension.to_string())),
                1 => return Ok(matching_langs[0]),
                _ => {
                    // Multiple languages share this extension - try shebang-based detection
                    // Try to read the file
                    let source = match std::fs::read(pb) {
                        Ok(content) => content,
                        Err(_) => {
                            // If we can't read the file, fall back to first candidate
                            return Ok(matching_langs[0]);
                        }
                    };

                    // Try each candidate language
                    for lang in &matching_langs {
                        if self.matches_language(&source, lang)? {
                            return Ok(lang);
                        }
                    }

                    // No match found; return first candidate as fallback
                    return Ok(matching_langs[0]);
                }
            }
        }

        Err(TopiaryConfigError::NoExtension(pb.clone()))
    }

    /// Checks if a source file matches a specific language using shebang detection.
    ///
    /// Uses find_injections to detect what language a shebang indicates.
    ///
    /// # Arguments
    ///
    /// * `source` - The file content as bytes
    /// * `lang` - The language to check against
    ///
    /// # Returns
    ///
    /// Returns `true` if the shebang indicates this language, `false` otherwise.
    #[allow(clippy::result_large_err)]
    fn matches_language(&self, source: &[u8], lang: &Language) -> TopiaryConfigResult<bool> {
        // Check if filetype detection is enabled for this language
        if !lang
            .config
            .filetype_detection
            .as_ref()
            .map(|d| d.enabled)
            .unwrap_or(false)
        {
            return Ok(false);
        }

        // Parse the file with the shell-shebang grammar
        let shebang_grammar = topiary_tree_sitter_shell_shebang::language();
        let shebang_grammar = topiary_tree_sitter_facade::Language::from(shebang_grammar);
        let mut parser = topiary_tree_sitter_facade::Parser::new()?;
        parser.set_language(&shebang_grammar)?;

        let tree = match parser.parse(source, None)? {
            Some(t) => t,
            None => return Ok(false),
        };

        // Load the injections query
        let query_source =
            include_str!("../../topiary-tree-sitter-shell-shebang/queries/injections.scm");
        let query = match topiary_tree_sitter_facade::Query::new(&shebang_grammar, query_source) {
            Ok(q) => q,
            Err(_) => return Ok(false),
        };

        // Use the injection system to detect the language
        let injections = topiary_core::find_injections(&tree, source, &query).map_err(|e| {
            TopiaryConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        // Check if any injection matches this language
        Ok(injections.iter().any(|inj| inj.language == lang.name))
    }

    #[allow(clippy::result_large_err)]
    fn parse_and_merge(sources: &[Source]) -> TopiaryConfigResult<(Self, RichTerm)> {
        let inputs = sources.iter().map(|s| s.clone().into());

        let mut program =
            Program::<CacheImpl>::new_from_inputs(inputs, std::io::stderr(), NullReporter {})?;

        let term = program.eval_full_for_export()?;

        let serde_config = SerdeConfiguration::deserialize(term.clone())?;

        Ok((serde_config.into(), term))
    }

    #[allow(clippy::result_large_err)]
    fn parse(source: Source) -> TopiaryConfigResult<(Self, RichTerm)> {
        let mut program = Program::<CacheImpl>::new_from_input(
            source.into(),
            std::io::stderr(),
            NullReporter {},
        )?;

        let term = program.eval_full_for_export()?;

        let serde_config = SerdeConfiguration::deserialize(term.clone())?;

        Ok((serde_config.into(), term))
    }
}

impl Default for Configuration {
    /// Return the built-in configuration
    // This is particularly useful for testing
    fn default() -> Self {
        let mut program = Program::<CacheImpl>::new_from_source(
            Source::Builtin
                .read()
                .expect("Evaluating the builtin configuration should be safe")
                .as_slice(),
            "built-in",
            std::io::empty(),
            NullReporter {},
        )
        .expect("Evaluating the builtin configuration should be safe");
        let term = program
            .eval_full_for_export()
            .expect("Evaluating the builtin configuration should be safe");
        let serde_config = SerdeConfiguration::deserialize(term)
            .expect("Evaluating the builtin configuration should be safe");

        serde_config.into()
    }
}

/// Convert `Serialisation` values into `HashMap`s, keyed on `Language::name`
impl From<&Configuration> for HashMap<String, Language> {
    fn from(config: &Configuration) -> Self {
        HashMap::from_iter(
            config
                .languages
                .iter()
                .map(|language| (language.name.clone(), language.clone())),
        )
    }
}

// Order-invariant equality; required for unit testing
impl PartialEq for Configuration {
    fn eq(&self, other: &Self) -> bool {
        let lhs: HashMap<String, Language> = self.into();
        let rhs: HashMap<String, Language> = other.into();

        lhs == rhs
    }
}

impl From<SerdeConfiguration> for Configuration {
    fn from(value: SerdeConfiguration) -> Self {
        let mut languages: Vec<_> = value
            .languages
            .into_iter()
            .map(|(name, config)| Language::new(name, config))
            .collect();

        // Sort languages alphabetically by name for deterministic ordering.
        // This ensures consistent fallback behavior when multiple languages
        // share the same extension (e.g., bash comes before zsh for .sh files).
        languages.sort_by(|a, b| a.name.cmp(&b.name));

        Self { languages }
    }
}

pub(crate) fn project_dirs() -> directories::ProjectDirs {
    directories::ProjectDirs::from("", "", "topiary")
        .expect("Could not access the OS's Home directory")
}
