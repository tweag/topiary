use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{Arc, Mutex},
};

use topiary::{Language, TopiaryQuery};

use crate::{error::CLIResult, io::InputFile};

/// `LanguageDefinition` contains the necessary language-related values that the Topiary API
/// expects to do its job
pub struct LanguageDefinition {
    pub query: TopiaryQuery,
    pub language: Language,
    pub grammar: tree_sitter_facade::Language,
}

/// Key type for the cache of language definitions
///
/// NOTE This is not public, as it is constructed from an `io::InputFile` reference
struct LanguageKey<'cfg> {
    language: &'cfg Language,
    query: PathBuf,
}

impl<'cfg> Hash for LanguageKey<'cfg> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.language.name.hash(state);
        self.query.hash(state);
    }
}

impl<'i, 'cfg> From<&'i InputFile<'cfg>> for LanguageKey<'cfg> {
    fn from(input: &'i InputFile<'cfg>) -> Self {
        todo!()
    }
}

/// Thread-safe language definition cache
pub struct LanguageDefinitionCache<'cfg>(Mutex<HashMap<LanguageKey<'cfg>, LanguageDefinition>>);

impl<'cfg> LanguageDefinitionCache<'cfg> {
    pub fn new() -> Arc<Self> {
        Arc::new(LanguageDefinitionCache(Mutex::new(HashMap::new())))
    }

    pub fn fetch<'k, T>(&mut self, key: &'k T) -> CLIResult<LanguageDefinition>
    where
        &'k T: Into<LanguageKey<'cfg>>,
    {
        self.0.lock()?;

        let key = key.into();
        todo!()
    }
}
