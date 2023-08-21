use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::PathBuf,
    sync::{Arc, RwLock},
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
#[derive(Eq, PartialEq)]
pub struct LanguageKey<'cfg> {
    language: &'cfg Language,
    query: PathBuf,
}

impl<'cfg> Hash for LanguageKey<'cfg> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.language.name.hash(state);
        self.query.hash(state);
    }
}

impl<'cfg> From<&'cfg InputFile<'cfg>> for LanguageKey<'cfg> {
    fn from(input: &'cfg InputFile<'cfg>) -> Self {
        Self {
            language: input.language(),
            query: input.query().into(),
        }
    }
}

/// Thread-safe language definition cache
pub struct LanguageDefinitionCache<'cfg>(
    RwLock<HashMap<LanguageKey<'cfg>, Arc<LanguageDefinition>>>,
);

impl<'cfg> LanguageDefinitionCache<'cfg> {
    pub fn new() -> Self {
        LanguageDefinitionCache(RwLock::new(HashMap::new()))
    }

    /// Fetch the language definition from the cache, populating if necessary, with thread-safety
    pub async fn fetch(&self, input: &'cfg InputFile<'cfg>) -> CLIResult<Arc<LanguageDefinition>> {
        let key = input.into();

        // Return the language definition from the cache, behind a read lock, if it exists...
        if let Some(lang_def) = self.0.read()?.get(&key) {
            return Ok(Arc::clone(lang_def));
        }

        // ...otherwise, fetch the language definition, to populate the cache behind a write lock
        let lang_def = Arc::new(input.to_language_definition().await?);
        self.0.write()?.insert(key, Arc::clone(&lang_def));
        Ok(lang_def)
    }
}
