use std::panic;
use topiary::{formatter, Configuration, FormatterResult, Operation};
use tree_sitter_facade::TreeSitter;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(js_name = topiaryInit)]
pub async fn topiary_init() -> Result<(), JsError> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "console_error_panic_hook")] {
            panic::set_hook(Box::new(console_error_panic_hook::hook));
        }
    }

    TreeSitter::init().await.map_err(|e| format_error(&*e))
}

#[wasm_bindgen]
pub async fn format(input: &str, query: &str) -> Result<String, JsError> {
    format_inner(input, query)
        .await
        .map_err(|e| format_error(&e))
}

async fn format_inner(input: &str, query: &str) -> FormatterResult<String> {
    let mut output = Vec::new();

    let configuration = Configuration::parse(query)?;
    let grammars = configuration.language.grammars().await?;

    formatter(
        &mut input.as_bytes(),
        &mut output,
        query,
        &configuration,
        &grammars,
        Operation::Format {
            skip_idempotence: true,
        },
    )?;

    Ok(String::from_utf8(output)?)
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
