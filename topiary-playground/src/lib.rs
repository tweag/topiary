#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;
#[cfg(target_arch = "wasm32")]
use topiary_core::{formatter, Configuration, FormatterResult, Language, Operation, TopiaryQuery};
#[cfg(target_arch = "wasm32")]
use tree_sitter_facade::TreeSitter;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
struct QueryState {
    language: Language,
    grammar: tree_sitter_facade::Language,
    query: TopiaryQuery,
}

#[cfg(target_arch = "wasm32")]
/// The query state is stored in a static variable, so the playground can reuse
/// it across multiple runs as long as it doesn't change.
static QUERY_STATE: Mutex<Option<QueryState>> = Mutex::new(None);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = topiaryInit)]
pub async fn topiary_init() -> Result<(), JsError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "console_error_panic_hook")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        }
    }

    TreeSitter::init().await
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(js_name = queryInit)]
pub async fn query_init(query_content: String, language_name: String) -> Result<(), JsError> {
    let language_normalized = language_name.replace('-', "_");
    let configuration = Configuration::parse_default_configuration()?;
    let language = configuration.get_language(language_normalized)?.clone();
    let grammar = language.grammar_wasm().await?;
    let query = TopiaryQuery::new(&grammar, &query_content)?;

    let mut guard = QUERY_STATE.lock().unwrap();

    *guard = Some(QueryState {
        language,
        grammar,
        query,
    });

    Ok(())
}

#[cfg(target_arch = "wasm32")]
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

#[cfg(target_arch = "wasm32")]
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
                &query_state.query,
                &query_state.language,
                &query_state.grammar,
                Operation::Format {
                    skip_idempotence: !check_idempotence,
                    tolerate_parsing_errors,
                },
            )?;

            Ok(String::from_utf8(output)?)
        }
        None => Err(topiary_core::FormatterError::Internal(
            "The query has not been initialized.".into(),
            None,
        )),
    }
}

#[cfg(target_arch = "wasm32")]
fn format_error(e: &dyn std::error::Error) -> JsError {
    let mut message: String = format!("{e}");
    let mut inner: &dyn std::error::Error = e;

    while let Some(source) = inner.source() {
        message += &format!("\nCause: {source}");
        inner = source;
    }

    JsError::new(&message)
}
