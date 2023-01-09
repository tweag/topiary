; Configuration
(#language! bash)

; Don't modify string literals
(string) @leaf

; Allow blank line before
; TODO Add directives here...
[
  (comment)
  (if_statement)
] @allow_blank_line_before

; Prepend spaces
; (Not "surround", to avoid extraneous spaces between delimiters)
; TODO Add keywords, etc., here...
[
  "if"
  "then"
  "elif"
  "else"
  "fi"
  (string)
  (command)
  (test_command)
] @prepend_space

;; Commands

; Space between command line arguments
(command
  argument: _ @append_space @prepend_space
)

; FIXME One command per line, modulo many exceptions:
; * Strings of && and ||
; * Pipelines
; * Subshells
; * As arguments to if, while, for, etc.
; * Others?...
;(command) @append_input_softline

;; Operators

; Ensure the negation operator is surrounded by spaces
(negated_command
  "!" @prepend_space @append_space
)

;; Conditionals

; Start conditional on a new line
[
  (if_statement)
  (elif_clause)
  (else_clause)
] @prepend_spaced_softline

; New line after "then" and start indent block
[
  (if_statement)
  (elif_clause)
] "then" @append_hardline @append_indent_start

; New line after "else" and start indent block
(else_clause "else" @append_hardline @append_indent_start)

; Keep the "if" and the "then" on the same line,
; inserting a delimiter when necessary
(if_statement
  (_) @append_delimiter
  ";"* @do_nothing
  "then"

  (#delimiter! ";")
)

; FIXME Can this be generalised from the above?
(elif_clause
  (_) @append_delimiter
  ";"* @do_nothing
  "then"

  (#delimiter! ";")
)

; Finish indent block at "fi", "else" or "elif"
(if_statement
  [
    "fi"
    (else_clause)
    (elif_clause)
  ] @prepend_indent_end @prepend_spaced_softline @append_hardline
)

;; Test Commands

(test_command
  (unary_expression) @prepend_space @append_space
)

; FIXME The binary_expression node is not being returned by Tree-Sitter
; See https://github.com/tweag/topiary/pull/155#issuecomment-1364143677
(test_command
  (binary_expression
     left: _ @prepend_space @append_space
     right: _ @prepend_space @append_space
  )
)
