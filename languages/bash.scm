; Configuration
(#language! bash)

; Don't modify string literals
(string) @leaf

; Allow blank line before
[
  (comment)
  (if_statement)
  (command)
  (list)
  (pipeline)
  ; TODO: etc.
] @allow_blank_line_before

; Prepend spaces
; (Not "surround", to avoid extraneous spaces between delimiters)
[
  "if"
  "then"
  "elif"
  "else"
  "fi"
  (string)
  (test_command)
  ; TODO: etc.
] @prepend_space

;; Commands

; NOTE "Command" is shorthand for a "unit of execution":
; * Simple commands (e.g., binaries, builtins, functions, etc.)
; * Command lists
; * Command pipelines
;
; That is: [(command) (list) (pipeline)], per the grammar

; FIXME I don't think it's possible to insert the necessary line
; continuations; or, at least, it's not possible to insert them only in
; a multi-line context. As such, all multi-line commands are forcibly
; collapsed on to a single line for now... See Issue #172

; One command per line in the following contexts:
; * Top-level
; * In any branch of a conditional
; * <TODO: etc.>
;
; NOTE Because "command" is such a pervasive and general concept, each
; context needs to be individually enumerated to account for exceptions.
(program [(command) (list) (pipeline)] @prepend_hardline)
(if_statement _ "then" [(command) (list) (pipeline)] @prepend_hardline)
(elif_clause _ "then" [(command) (list) (pipeline)] @prepend_hardline)
(else_clause [(command) (list) (pipeline)] @prepend_hardline)

; Surround command list and pipeline delimiters with spaces
(list ["&&" "||"] @append_space @prepend_space)
(pipeline ["|" "|&"] @append_space @prepend_space)

; Prepend the asynchronous operator with a space
; NOTE If I'm not mistaken, this can interpose two "commands" -- like a
; delimiter -- but I've never seen this form in the wild
(_ [(command) (list) (pipeline)] . "&" @prepend_space)

; Space between command line arguments
(command
  argument: _ @append_space @prepend_space
)

;; Operators

; Ensure the negation operator is surrounded by spaces
; NOTE This is a syntactic requirement
(negated_command
  "!" @prepend_space @append_space
)

;; Conditionals

; Start conditional on a new line
[
  (if_statement)
  (elif_clause)
  (else_clause)
] @prepend_hardline

; New line after "then" and start indent block
[
  (if_statement)
  (elif_clause)
] "then" @append_hardline @append_indent_start

; New line after "else" and start indent block
(else_clause "else" @append_hardline @append_indent_start)

; Keep the "if" and the "then" on the same line,
; inserting a delimiter when necessary
(_
  (_) @prepend_space @append_delimiter
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
