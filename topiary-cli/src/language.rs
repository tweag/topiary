use std::{
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
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

/// Thread-safe language definition cache
pub struct LanguageDefinitionCache(RwLock<HashMap<u64, Arc<LanguageDefinition>>>);

impl LanguageDefinitionCache {
    pub fn new() -> Self {
        LanguageDefinitionCache(RwLock::new(HashMap::new()))
    }

    /// Fetch the language definition from the cache, populating if necessary, with thread-safety
    // FIXME Locking is not working as expected. The read/write locks ensure atomicity -- so it's
    // not UN-thread-safe -- but many threads/futures can pass the read lock and go into the write
    // branch. Using a `Mutex` over the whole `HashMap`, rather than a `RwLock`, breaks the `Send`
    // constraint of the async scope...
    pub async fn fetch<'i>(&self, input: &'i InputFile<'i>) -> CLIResult<Arc<LanguageDefinition>> {
        // There's no need to store the input's identifying information (language name and query)
        // in the key, so we use its hash directly. This side-steps any awkward lifetime issues.
        let key = {
            let mut hash = DefaultHasher::new();
            input.language().name.hash(&mut hash);
            input.query().hash(&mut hash);

            hash.finish()
        };

        // Return the language definition from the cache, behind a read lock, if it exists...
        if let Some(lang_def) = self.0.read()?.get(&key) {
            log::debug!(
                "Cache {:p}: Hit at {:#016x} ({}, {})",
                self,
                key,
                input.language(),
                input.query().file_name().unwrap().to_string_lossy()
            );

            return Ok(Arc::clone(lang_def));
        }

        // ...otherwise, fetch the language definition, to populate the cache behind a write lock
        let lang_def = Arc::new(input.to_language_definition().await?);
        self.0.write()?.insert(key, Arc::clone(&lang_def));

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
