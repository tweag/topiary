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

; Surround spaces
; TODO Add keywords, etc., here...
[
  "if"
  "then"
  "elif"
  "else"
  "fi"
  (string)
  (test_command)
] @prepend_space @append_space

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

; Ensure there's a space after the negation operator
(negated_command
  "!" @append_space
)

;; Conditionals

; New line after "then" and start indent block
(if_statement
  "then" @append_hardline @append_indent_start
) @prepend_spaced_softline

; FIXME Can this be generalised from the above?
(elif_clause
  "then" @append_hardline @append_indent_start
) @prepend_spaced_softline

(else_clause
  "else" @append_hardline @append_indent_start
) @prepend_spaced_softline

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

; FIXME This doesn't do anything :(
(test_command
  (binary_expression
     left: _ @prepend_space @append_space
     right: _ @prepend_space @append_space
  )
)
