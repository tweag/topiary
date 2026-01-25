/**
 * Tree-sitter grammar for generic shell files with shebang-based injection
 * 
 * Parses files like:
 *   #!/bin/bash
 *   echo "hello world"
 *   for i in 1 2 3; do
 *     echo $i
 *   done
 * 
 * Uses tree-sitter injection mechanism to:
 * 1. Parse the shebang line to detect interpreter (bash/zsh)
 * 2. Mark the rest of the file for injection with the detected language
 */

module.exports = grammar({
  name: 'generic_shell',

  rules: {
    // Root: optional shebang line followed by script content
    source_file: $ => seq(
      optional($.shebang_line),
      optional($.script_content)
    ),

    // A shebang line - parse it to extract interpreter info
    shebang_line: $ => choice(
      // Direct interpreter: #!/bin/bash
      seq(
        token(prec(1, '#!')),
        optional($.whitespace),
        $.direct_interpreter,
        optional($.args),
        '\n'
      ),
      // Via env: #!/usr/bin/env bash
      seq(
        token(prec(1, '#!')),
        optional($.whitespace),
        $.env_path,
        $.whitespace,
        optional($.env_flags),
        optional($.whitespace),
        $.env_interpreter,
        optional($.args),
        '\n'
      )
    ),

    // Explicit whitespace node for topiary to capture
    whitespace: $ => /[ \t]+/,

    // Direct path to interpreter: /bin/bash, /usr/bin/zsh
    // Capture the full path as well as just the interpreter name
    direct_interpreter: $ => seq(
      $.interpreter_path,  // Path prefix
      $.interpreter_name
    ),
    
    // Path to the interpreter (/bin/, /usr/bin/, etc)
    interpreter_path: $ => /[^ \t\n]*\//,
    
    // Just the interpreter name (bash or zsh)
    interpreter_name: $ => /(bash|zsh)/,

    // Path to env command
    env_path: $ => /[^ \t\n]*\/env/,

    // Env flags like -S, -i
    env_flags: $ => /-[^ \t\n]+/,

    // Interpreter name after env: bash, zsh
    env_interpreter: $ => /(bash|zsh)/,

    // Additional arguments
    args: $ => /[ \t]+[^\n]*/,

    // All content after the shebang - this is what gets injected
    script_content: $ => /(.|\n)+/,
  }
});
