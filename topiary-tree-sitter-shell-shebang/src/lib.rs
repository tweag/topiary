//! Tree-sitter grammar for parsing shebang lines
//!
//! This grammar can parse shebang lines to extract interpreter information,
//! which is useful for disambiguating files with ambiguous extensions.

use tree_sitter::Language;

extern "C" {
    fn tree_sitter_shell_shebang() -> Language;
}

/// Get the tree-sitter Language for parsing shell shebang lines
pub fn language() -> Language {
    unsafe { tree_sitter_shell_shebang() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_load_grammar() {
        let _language = language();
    }
}
