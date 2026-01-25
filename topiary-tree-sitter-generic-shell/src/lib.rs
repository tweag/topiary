//! Tree-sitter grammar for generic shell files
//!
//! This grammar can parse shebang lines to extract interpreter information (bash, zsh, etc.),
//! which is useful for disambiguating files with ambiguous extensions like `.sh`.

use tree_sitter::Language;

extern "C" {
    fn tree_sitter_generic_shell() -> Language;
}

/// Get the tree-sitter Language for parsing generic shell files
pub fn language() -> Language {
    unsafe { tree_sitter_generic_shell() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let _language = language();
    }
}
