/// We create a private module for all the wasm code, and then export every symbol in that module.
/// This prevents us from having to declare the `target_arch` for every symbol.

#[cfg(target_arch = "wasm32")]
pub use wasm_mod::*;

#[cfg(target_arch = "wasm32")]
mod wasm_mod {
    use std::sync::Mutex;
    use topiary::{formatter, FormatterResult, Language, Operation, TopiaryQuery};
    use topiary_config::Configuration;
    use topiary_tree_sitter_facade::TreeSitter;
    use wasm_bindgen::prelude::*;

    struct QueryState {
        language: Language,
    }

    /// The query state is stored in a static variable, so the playground can reuse
    /// it across multiple runs as long as it doesn't change.
    static QUERY_STATE: Mutex<Option<QueryState>> = Mutex::new(None);

    #[wasm_bindgen(js_name = topiaryInit)]
    #[cfg(not(feature = "console_error_panic_hook"))]
    pub async fn topiary_init() -> Result<(), JsError> {
        TreeSitter::init().await
    }

    #[wasm_bindgen(js_name = topiaryInit)]
    #[cfg(feature = "console_error_panic_hook")]
    pub async fn topiary_init() -> Result<(), JsError> {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        TreeSitter::init().await
    }

    #[wasm_bindgen(js_name = queryInit)]
    pub async fn query_init(query_content: String, language_name: String) -> Result<(), JsError> {
        let language_normalized = language_name.replace('-', "_");
        let configuration = Configuration::default();
        let language = configuration.get_language(language_normalized)?.clone();
        let grammar = language.grammar().await?;
        let query = TopiaryQuery::new(&grammar, &query_content)?;
        let mut guard = QUERY_STATE.lock().unwrap();
        let language = Language {
            name: language.name,
            query,
            grammar,
            indent: language.indent,
        };

        *guard = Some(QueryState { language });

        Ok(())
    }

    #[wasm_bindgen]
    pub async fn format(
        input: &str,
        check_idempotence: bool,
        tolerate_parsing_errors: bool,
    ) -> Result<String, JsError> {
        format_inner(input, check_idempotence, tolerate_parsing_errors)
            .await
            .map_err(|e| format_error(&e))
    }

    async fn format_inner(
        input: &str,
        check_idempotence: bool,
        tolerate_parsing_errors: bool,
    ) -> FormatterResult<String> {
        let mut output = Vec::new();

        let mut guard = QUERY_STATE.lock().unwrap();

        match &mut *guard {
            Some(query_state) => {
                formatter(
                    &mut input.as_bytes(),
                    &mut output,
                    &query_state.language,
                    Operation::Format {
                        skip_idempotence: !check_idempotence,
                        tolerate_parsing_errors,
                    },
                )?;

                Ok(String::from_utf8(output)?)
            }
            None => Err(topiary::FormatterError::Internal(
                "The query has not been initialized.".into(),
                None,
            )),
        }
    }

    fn format_error(e: &dyn std::error::Error) -> JsError {
        let mut message: String = format!("{e}");
        let mut inner: &dyn std::error::Error = e;

        while let Some(source) = inner.source() {
            message += &format!("\nCause: {source}");
            inner = source;
        }

        JsError::new(&message)
    }
}
