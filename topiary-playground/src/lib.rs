#[cfg(target_arch = "wasm32")]
use js_sys::Promise;
#[cfg(target_arch = "wasm32")]
use std::sync::Mutex;
#[cfg(target_arch = "wasm32")]
use topiary::{formatter, Configuration, FormatterResult, Language, Operation, TopiaryQuery};
#[cfg(target_arch = "wasm32")]
use tree_sitter_facade::TreeSitter;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::future_to_promise;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

// #[cfg(target_arch = "wasm32")]
// thread_local! {
//     static LANGUAGE: RefCell<Option<Language>> = RefCell::new(None);
//     static GRAMMAR: RefCell<Option<tree_sitter_facade::Language>> = RefCell::new(None);
//     static QUERY: RefCell<Option<TopiaryQuery>> = RefCell::new(None);
// }

#[cfg(target_arch = "wasm32")]
struct QueryState {
    language: Language,
    grammar: tree_sitter_facade::Language,
    query: TopiaryQuery,
}

#[cfg(target_arch = "wasm32")]
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
pub async fn query_init(query_content: String, language_name: String) -> Promise {
    let fut = async { query_init_wasm(query_content, language_name) };

    future_to_promise(fut.await)
}

#[cfg(target_arch = "wasm32")]
async fn query_init_wasm(query_content: String, language_name: String) -> Result<JsValue, JsValue> {
    query_init_inner(query_content, language_name)
        .await
        .map(|_| JsValue::UNDEFINED)
        .map_err(|e| e.into())
}

#[cfg(target_arch = "wasm32")]
async fn query_init_inner(query_content: String, language_name: String) -> Result<(), JsError> {
    use topiary::FormatterError;

    console::log_3(
        &"query_init:".into(),
        &query_content.clone().into(),
        &language_name.clone().into(),
    );

    let language_normalized = language_name.replace('-', "_");
    let configuration = Configuration::parse_default_configuration()?;
    console::log_1(&"config set.".into());
    let language = configuration.get_language(language_normalized)?.clone();
    console::log_1(&"lang set.".into());
    let grammar = language.grammar_wasm().await?;
    console::log_1(&"grammar set.".into());
    let query = TopiaryQuery::new(&grammar, &query_content)?;
    console::log_1(&"query set.".into());

    let mut guard = QUERY_STATE.lock().unwrap();

    *guard = Some(QueryState {
        language,
        grammar,
        query,
    });

    console::log_1(&"Query state updated".into());

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
    use topiary::FormatterError;

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
        None => Err(FormatterError::Internal(
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
