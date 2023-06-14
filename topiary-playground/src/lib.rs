#[cfg(target_arch = "wasm32")]
use topiary::{formatter, Configuration, FormatterResult, Operation};
#[cfg(target_arch = "wasm32")]
use tree_sitter_facade::TreeSitter;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
#[wasm_bindgen]
pub async fn format(
    input: &str,
    query: &str,
    language: &str,
    check_idempotence: bool,
) -> Result<String, JsError> {
    let language_normalized = language.replace('-', "_");
    format_inner(
        input,
        query,
        language_normalized.as_str(),
        check_idempotence,
    )
    .await
    .map_err(|e| format_error(&e))
}

#[cfg(target_arch = "wasm32")]
async fn format_inner(
    input: &str,
    query: &str,
    language_name: &str,
    check_idempotence: bool,
) -> FormatterResult<String> {
    let mut output = Vec::new();

    let configuration = Configuration::parse_default_configuration()?;
    let language = configuration.get_language(language_name)?;
    let grammars = language.grammars_wasm().await?;

    formatter(
        &mut input.as_bytes(),
        &mut output,
        query,
        language,
        &grammars,
        Operation::Format {
            skip_idempotence: !check_idempotence,
        },
    )?;

    Ok(String::from_utf8(output)?)
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
