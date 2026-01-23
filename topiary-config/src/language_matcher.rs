//! Language matching strategies for file type detection.
//!
//! This module provides a trait-based system for disambiguating languages when multiple
//! languages share the same file extension. The primary use case is distinguishing between
//! shell dialects (bash, zsh) that often use the same `.sh` extension.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// A trait for language-specific matching strategies.
///
/// Implementations of this trait provide custom logic to determine if a file
/// belongs to a particular language, beyond simple extension matching.
pub trait LanguageMatcher {
    /// Returns true if the file matches this language's characteristics.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the file being analyzed
    ///
    /// # Returns
    ///
    /// * `Some(true)` if the file definitively matches this language
    /// * `Some(false)` if the file definitively does not match this language
    /// * `None` if the matcher cannot determine (e.g., file cannot be read, no shebang)
    fn matches(&self, path: &Path) -> Option<bool>;
}

/// Extracts the shebang line from a file, if present.
///
/// # Arguments
///
/// * `path` - The path to the file to read
///
/// # Returns
///
/// * `Some(String)` containing the first line if it starts with `#!`
/// * `None` if the file has no shebang, cannot be read, or is empty
pub fn extract_shebang(path: &Path) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();

    reader.read_line(&mut first_line).ok()?;

    if first_line.starts_with("#!") {
        Some(first_line.trim().to_string())
    } else {
        None
    }
}

/// Checks if a shebang line references a specific interpreter.
///
/// This function handles common shebang patterns:
/// - Direct paths: `#!/bin/bash`, `#!/usr/bin/zsh`
/// - Env indirection: `#!/usr/bin/env bash`, `#!/usr/bin/env -S zsh`
/// - With arguments: `#!/bin/bash -e`, `#!/usr/bin/env bash -x`
///
/// # Arguments
///
/// * `shebang` - The shebang line (including `#!` prefix)
/// * `interpreter` - The interpreter name to search for (e.g., "bash", "zsh")
///
/// # Returns
///
/// * `true` if the shebang references the given interpreter
/// * `false` otherwise
pub fn shebang_matches_interpreter(shebang: &str, interpreter: &str) -> bool {
    // Remove the #! prefix and split into tokens
    let tokens: Vec<&str> = shebang
        .trim_start_matches("#!")
        .split_whitespace()
        .collect();

    if tokens.is_empty() {
        return false;
    }

    // Check if first token ends with the interpreter name
    // e.g., /bin/bash, /usr/bin/bash, bash
    if tokens[0].ends_with(interpreter) {
        return true;
    }

    // Check for env pattern: #!/usr/bin/env [flags] <interpreter>
    if tokens[0].ends_with("env") {
        // Skip optional env flags (like -S, -i, etc.)
        for token in tokens.iter().skip(1) {
            // Skip flags starting with -
            if token.starts_with('-') {
                continue;
            }
            // First non-flag token should be the interpreter
            return token.ends_with(interpreter);
        }
    }

    false
}

/// A matcher for bash scripts based on shebang detection.
pub struct BashMatcher;

impl LanguageMatcher for BashMatcher {
    fn matches(&self, path: &Path) -> Option<bool> {
        let shebang = extract_shebang(path)?;
        Some(shebang_matches_interpreter(&shebang, "bash"))
    }
}

/// A matcher for zsh scripts based on shebang detection.
pub struct ZshMatcher;

impl LanguageMatcher for ZshMatcher {
    fn matches(&self, path: &Path) -> Option<bool> {
        let shebang = extract_shebang(path)?;
        Some(shebang_matches_interpreter(&shebang, "zsh"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shebang_matches_direct_path() {
        assert!(shebang_matches_interpreter("#!/bin/bash", "bash"));
        assert!(shebang_matches_interpreter("#!/usr/bin/bash", "bash"));
        assert!(shebang_matches_interpreter("#!/usr/local/bin/bash", "bash"));
        assert!(!shebang_matches_interpreter("#!/bin/bash", "zsh"));
    }

    #[test]
    fn test_shebang_matches_env() {
        assert!(shebang_matches_interpreter("#!/usr/bin/env bash", "bash"));
        assert!(shebang_matches_interpreter("#!/usr/bin/env zsh", "zsh"));
        assert!(shebang_matches_interpreter("#!/bin/env bash", "bash"));
        assert!(!shebang_matches_interpreter("#!/usr/bin/env bash", "zsh"));
    }

    #[test]
    fn test_shebang_matches_with_args() {
        assert!(shebang_matches_interpreter("#!/bin/bash -e", "bash"));
        assert!(shebang_matches_interpreter(
            "#!/usr/bin/env bash -x",
            "bash"
        ));
        assert!(shebang_matches_interpreter("#!/bin/zsh -f", "zsh"));
    }

    #[test]
    fn test_shebang_matches_env_with_flags() {
        assert!(shebang_matches_interpreter(
            "#!/usr/bin/env -S bash",
            "bash"
        ));
        assert!(shebang_matches_interpreter("#!/usr/bin/env -i zsh", "zsh"));
        assert!(shebang_matches_interpreter(
            "#!/usr/bin/env -S bash -e",
            "bash"
        ));
    }

    #[test]
    fn test_shebang_no_match() {
        assert!(!shebang_matches_interpreter("#!/bin/sh", "bash"));
        assert!(!shebang_matches_interpreter("#!/usr/bin/python", "bash"));
        assert!(!shebang_matches_interpreter("", "bash"));
        assert!(!shebang_matches_interpreter("#comment", "bash"));
    }

    #[test]
    fn test_shebang_edge_cases() {
        // No space after #!
        assert!(shebang_matches_interpreter("#!/bin/bash", "bash"));
        // Multiple spaces
        assert!(shebang_matches_interpreter("#!/usr/bin/env  bash", "bash"));
        // Tab characters
        assert!(shebang_matches_interpreter("#!/usr/bin/env\tbash", "bash"));
    }
}
