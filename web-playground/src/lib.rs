use std::panic;
use topiary::{formatter, Configuration, Language, Operation};
use tree_sitter_facade::TreeSitter;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(js_name = topiaryInit)]
pub async fn topiary_init() {
    cfg_if::cfg_if! {
        if #[cfg(feature = "console_error_panic_hook")] {
            panic::set_hook(Box::new(console_error_panic_hook::hook));
        }
    }

    TreeSitter::init().await;
}

#[wasm_bindgen]
pub async fn format(input: &str, query: &str) -> Result<String, JsError> {
    let mut output = Vec::new();

    let mut configuration = Configuration::parse(&query)?;
    configuration.language = Language::Json;

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
