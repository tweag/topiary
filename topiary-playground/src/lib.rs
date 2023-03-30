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
pub async fn format(input: &str, query: &str) -> Result<String, JsError> {
    format_inner(input, query)
        .await
        .map_err(|e| format_error(&e))
}

#[cfg(target_arch = "wasm32")]
async fn format_inner(input: &str, query: &str) -> FormatterResult<String> {
    let mut output = Vec::new();

    let configuration = Configuration::parse(query)?;
    let grammars = configuration.language.grammars_wasm().await?;

    formatter(
        &mut input.as_bytes(),
        &mut output,
        query,
        &configuration,
        &grammars,
        Operation::Format {
            check_input_exhaustivity: false,
            skip_idempotence: true,
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
