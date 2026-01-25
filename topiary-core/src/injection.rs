/// Tree-sitter injection support for Topiary
///
/// This module implements language injection, allowing a parent grammar to
/// inject child grammars for specific portions of the parse tree.
///
/// Example: A shell_shebang grammar can inject bash/zsh for script content.
use streaming_iterator::StreamingIterator;

use crate::{FormatterError, FormatterResult};
use topiary_tree_sitter_facade::{Query, QueryCursor, QueryMatch, Range, Tree};

/// Represents a detected language injection point
#[derive(Debug, Clone)]
pub struct Injection {
    /// The language to inject (e.g., "bash", "zsh", "javascript")
    pub language: String,

    /// The byte range to re-parse with the injected language
    pub content_range: Range,
}

/// Parse injection queries and extract injection points from a tree
///
/// This function looks for captures named:
/// - `@injection.language` - specifies which language to inject
/// - `@injection.content` - specifies the content to re-parse
///
/// # Arguments
/// * `tree` - The parent tree to search for injections
/// * `source` - The source code bytes
/// * `injections_query` - The injection query (typically from injections.scm)
///
/// # Returns
/// A vector of `Injection` structs, one for each injection point found
///
/// # Example
/// ```scheme
/// ; In injections.scm
/// (env_interpreter) @injection.language
/// (script_content) @injection.content
/// ```
pub fn find_injections(
    tree: &Tree,
    source: &[u8],
    injections_query: &Query,
) -> FormatterResult<Vec<Injection>> {
    let mut cursor = QueryCursor::new();
    let root_node = tree.root_node();

    // Collect all captures
    let mut language_captures: Vec<(usize, String)> = Vec::new();
    let mut content_captures: Vec<(usize, Range)> = Vec::new();

    // Get capture names from the query
    let capture_names: Vec<&str> = injections_query.capture_names();

    // Run the query using the correct API
    let mut query_matches = injections_query.matches(&root_node, source, &mut cursor);

    #[allow(clippy::while_let_on_iterator)]
    while let Some(query_match) = query_matches.next() {
        let pattern_index = query_match.pattern_index();

        // First, check if this pattern has injection.language set via #set! predicate
        let mut lang_from_property: Option<String> = None;
        for (key, value) in injections_query.property_settings(pattern_index) {
            if key == "injection.language" {
                if let Some(v) = value {
                    lang_from_property = Some(v.to_string());
                }
                break;
            }
        }

        // Process captures for this match
        for capture in query_match.captures() {
            let capture_name = capture.name(&capture_names);

            if capture_name == "injection.language" {
                // Extract the language name from the captured node's text
                let node = capture.node();
                let text = node
                    .utf8_text(source)
                    .map_err(|e| {
                        FormatterError::Injection(
                            format!("Invalid UTF-8 in injection.language capture: {}", e),
                            None,
                        )
                    })?
                    .to_string();

                language_captures.push((pattern_index, text));
            } else if capture_name == "injection.content" {
                let node = capture.node();
                let content_range = Range::new(
                    node.start_byte(),
                    node.end_byte(),
                    &node.start_position(),
                    &node.end_position(),
                );
                content_captures.push((pattern_index, content_range));
            }
        }

        // If we found a language via #set! property, add it to captures
        if let Some(lang) = lang_from_property {
            language_captures.push((pattern_index, lang));
        }
    }

    // Match language captures with content captures
    // In tree-sitter injections, language detection and content marking are separate:
    // - Language patterns identify what language to inject (can be multiple patterns)
    // - Content patterns mark what regions to inject
    // For shebang-style injections, we have one language for the entire file
    let mut injections = Vec::new();

    // If we have any language detection and any content regions, pair them up
    if !language_captures.is_empty() && !content_captures.is_empty() {
        // Use the first language found (for single-file shebangs)
        let language = &language_captures[0].1;

        // Create injections for all content regions with this language
        for (_content_pattern, content_range) in content_captures {
            injections.push(Injection {
                language: language.clone(),
                content_range,
            });
        }
    }

    Ok(injections)
}

#[cfg(test)]
mod tests {
    use super::*;
    use topiary_tree_sitter_facade::Parser;

    #[test]
    fn test_injection_struct_creation() {
        // Test that we can create an Injection struct
        let injection = Injection {
            language: "bash".to_string(),
            content_range: Range::new(0, 10, &Default::default(), &Default::default()),
        };

        assert_eq!(injection.language, "bash");
        assert_eq!(injection.content_range.start_byte(), 0);
        assert_eq!(injection.content_range.end_byte(), 10);
    }

    #[test]
    fn test_find_injections_no_matches() {
        // Test with a simple grammar that has no injection matches
        let shebang_grammar = topiary_tree_sitter_shell_shebang::language();
        let shebang_grammar = topiary_tree_sitter_facade::Language::from(shebang_grammar);

        let mut parser = Parser::new().unwrap();
        parser.set_language(&shebang_grammar).unwrap();

        // Parse a file without a proper shebang
        let source = b"echo hello";
        let tree = parser.parse(source, None).unwrap().unwrap();

        // Load the injections query
        let query_source =
            include_str!("../../topiary-tree-sitter-shell-shebang/queries/injections.scm");
        let query = Query::new(&shebang_grammar, query_source).unwrap();

        let injections = find_injections(&tree, source, &query).unwrap();

        assert_eq!(injections.len(), 0);
    }

    #[test]
    fn test_find_injections_bash_shebang() {
        // Test bash shebang detection via env
        let shebang_grammar = topiary_tree_sitter_shell_shebang::language();
        let shebang_grammar = topiary_tree_sitter_facade::Language::from(shebang_grammar);

        let mut parser = Parser::new().unwrap();
        parser.set_language(&shebang_grammar).unwrap();

        let source = b"#!/usr/bin/env bash\necho hello\n";
        let tree = parser.parse(source, None).unwrap().unwrap();

        let query_source =
            include_str!("../../topiary-tree-sitter-shell-shebang/queries/injections.scm");
        let query = Query::new(&shebang_grammar, query_source).unwrap();

        let injections = find_injections(&tree, source, &query).unwrap();

        // Should find one injection
        assert_eq!(injections.len(), 1, "Expected 1 injection");
        assert_eq!(injections[0].language, "bash");

        // Content should be everything after the shebang line
        let start = injections[0].content_range.start_byte() as usize;
        let end = injections[0].content_range.end_byte() as usize;
        let content = &source[start..end];
        assert_eq!(content, b"echo hello\n");
    }

    #[test]
    fn test_find_injections_zsh_shebang() {
        // Test zsh shebang detection via env
        let shebang_grammar = topiary_tree_sitter_shell_shebang::language();
        let shebang_grammar = topiary_tree_sitter_facade::Language::from(shebang_grammar);

        let mut parser = Parser::new().unwrap();
        parser.set_language(&shebang_grammar).unwrap();

        let source = b"#!/usr/bin/env zsh\nfor i in 1 2 3; do echo $i; done\n";
        let tree = parser.parse(source, None).unwrap().unwrap();

        let query_source =
            include_str!("../../topiary-tree-sitter-shell-shebang/queries/injections.scm");
        let query = Query::new(&shebang_grammar, query_source).unwrap();

        let injections = find_injections(&tree, source, &query).unwrap();

        // Should find one injection with zsh
        assert_eq!(injections.len(), 1);
        assert_eq!(injections[0].language, "zsh");

        let start = injections[0].content_range.start_byte() as usize;
        let end = injections[0].content_range.end_byte() as usize;
        let content = &source[start..end];
        assert_eq!(content, b"for i in 1 2 3; do echo $i; done\n");
    }

    #[test]
    fn test_find_injections_direct_zsh_path() {
        // Test zsh shebang with direct path
        let shebang_grammar = topiary_tree_sitter_shell_shebang::language();
        let shebang_grammar = topiary_tree_sitter_facade::Language::from(shebang_grammar);

        let mut parser = Parser::new().unwrap();
        parser.set_language(&shebang_grammar).unwrap();

        let source = b"#!/usr/bin/zsh\necho test\n";
        let tree = parser.parse(source, None).unwrap().unwrap();

        let query_source =
            include_str!("../../topiary-tree-sitter-shell-shebang/queries/injections.scm");
        let query = Query::new(&shebang_grammar, query_source).unwrap();

        let injections = find_injections(&tree, source, &query).unwrap();

        assert_eq!(injections.len(), 1);
        assert_eq!(injections[0].language, "zsh");
    }

    #[test]
    fn test_find_injections_empty_script() {
        // Test shebang with no content
        let shebang_grammar = topiary_tree_sitter_shell_shebang::language();
        let shebang_grammar = topiary_tree_sitter_facade::Language::from(shebang_grammar);

        let mut parser = Parser::new().unwrap();
        parser.set_language(&shebang_grammar).unwrap();

        let source = b"#!/bin/bash\n";
        let tree = parser.parse(source, None).unwrap().unwrap();

        let query_source =
            include_str!("../../topiary-tree-sitter-shell-shebang/queries/injections.scm");
        let query = Query::new(&shebang_grammar, query_source).unwrap();

        let injections = find_injections(&tree, source, &query).unwrap();

        // Should find bash language but no content (empty script_content node)
        // The grammar might still create an empty script_content node
        if !injections.is_empty() {
            assert_eq!(injections[0].language, "bash");
        }
    }

    #[test]
    fn test_find_injections_direct_bash_path() {
        // Test direct bash path (not via env)
        let shebang_grammar = topiary_tree_sitter_shell_shebang::language();
        let shebang_grammar = topiary_tree_sitter_facade::Language::from(shebang_grammar);

        let mut parser = Parser::new().unwrap();
        parser.set_language(&shebang_grammar).unwrap();

        let source = b"#!/bin/bash\necho hello world\n";
        let tree = parser.parse(source, None).unwrap().unwrap();

        let query_source =
            include_str!("../../topiary-tree-sitter-shell-shebang/queries/injections.scm");
        let query = Query::new(&shebang_grammar, query_source).unwrap();

        let injections = find_injections(&tree, source, &query).unwrap();

        assert_eq!(injections.len(), 1);
        assert_eq!(injections[0].language, "bash");

        let start = injections[0].content_range.start_byte() as usize;
        let end = injections[0].content_range.end_byte() as usize;
        let content = &source[start..end];
        assert_eq!(content, b"echo hello world\n");
    }

    #[test]
    fn test_find_injections_with_args() {
        // Test shebang with arguments
        let shebang_grammar = topiary_tree_sitter_shell_shebang::language();
        let shebang_grammar = topiary_tree_sitter_facade::Language::from(shebang_grammar);

        let mut parser = Parser::new().unwrap();
        parser.set_language(&shebang_grammar).unwrap();

        let source = b"#!/usr/bin/env -S bash -x\nset -e\n";
        let tree = parser.parse(source, None).unwrap().unwrap();

        let query_source =
            include_str!("../../topiary-tree-sitter-shell-shebang/queries/injections.scm");
        let query = Query::new(&shebang_grammar, query_source).unwrap();

        let injections = find_injections(&tree, source, &query).unwrap();

        assert_eq!(injections.len(), 1);
        assert_eq!(injections[0].language, "bash");

        let start = injections[0].content_range.start_byte() as usize;
        let end = injections[0].content_range.end_byte() as usize;
        let content = &source[start..end];
        assert_eq!(content, b"set -e\n");
    }
}
