use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    sync::Arc,
};

use tokio::sync::Mutex;
use topiary::{Language, TopiaryQuery};

use crate::{error::CLIResult, io::InputFile};

/// `LanguageDefinition` contains the necessary language-related values that the Topiary API
/// expects to do its job
pub struct LanguageDefinition {
    pub query: TopiaryQuery,
    pub language: Language,
    pub grammar: tree_sitter_facade::Language,
}

/// Thread-safe language definition cache
pub struct LanguageDefinitionCache(Mutex<HashMap<u64, Arc<LanguageDefinition>>>);

impl LanguageDefinitionCache {
    pub fn new() -> Self {
        LanguageDefinitionCache(Mutex::new(HashMap::new()))
    }

    /// Fetch the language definition from the cache, populating if necessary, with thread-safety
    pub async fn fetch<'i>(&self, input: &'i InputFile<'i>) -> CLIResult<Arc<LanguageDefinition>> {
        // There's no need to store the input's identifying information (language name and query)
        // in the key, so we use its hash directly. This side-steps any awkward lifetime issues.
        let key = {
            let mut hash = DefaultHasher::new();
            input.language().name.hash(&mut hash);
            input.query().hash(&mut hash);

            hash.finish()
        };

        // Lock the entire `HashMap` on access. (This may seem blunt, but is necessary for the
        // correct behaviour when we have near-simultaneous cache access; see issue #605.)
        let mut cache = self.0.lock().await;

        // Return the language definition from the cache, if it exists...
        if let Some(lang_def) = cache.get(&key) {
            log::debug!(
                "Cache {:p}: Hit at {:#016x} ({}, {})",
                self,
                key,
                input.language(),
                input.query().file_name().unwrap().to_string_lossy()
            );

            return Ok(Arc::clone(lang_def));
        }

        // ...otherwise, fetch the language definition, to populate the cache
        let lang_def = Arc::new(input.to_language_definition().await?);
        cache.insert(key, Arc::clone(&lang_def));

        log::debug!(
            "Cache {:p}: Insert at {:#016x} ({}, {})",
            self,
            key,
            input.language(),
            input.query().file_name().unwrap().to_string_lossy()
        );

        Ok(lang_def)
    }
}
