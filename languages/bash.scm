; Configuration
(#language! bash)

; FIXME Blank line spacing around major syntactic blocks is not correct.
; Some blank lines are getting consumed unexpectedly in the output.

; Don't modify string literals
(string) @leaf

; Allow blank line before
[
  (comment)
  (if_statement)
  (command)
  (list)
  (pipeline)
  (for_statement)
  (c_style_for_statement)
  ; TODO: etc.
] @allow_blank_line_before

; Surround with spaces
[
  "if"
  "then"
  "elif"
  "else"
  "fi"
  "for"
  "select"
  "in"
  "do"
  "done"
  (string)
  ; TODO: etc.
] @append_space @prepend_space

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
; * Within loops
; * <TODO: etc.>
;
; NOTE Because "command" is such a pervasive and general concept, each
; context needs to be individually enumerated to account for exceptions.
(program [(command) (list) (pipeline)] @prepend_hardline)
(if_statement . _ "then" [(command) (list) (pipeline)] @prepend_hardline)
(elif_clause . _ "then" [(command) (list) (pipeline)] @prepend_hardline)
(else_clause . "else" [(command) (list) (pipeline)] @prepend_hardline)
(do_group . "do" [(command) (list) (pipeline)] @prepend_hardline)

; Surround command list and pipeline delimiters with spaces
(list ["&&" "||"] @append_space @prepend_space)
(pipeline ["|" "|&"] @append_space @prepend_space)

; Prepend the asynchronous operator with a space
; NOTE If I'm not mistaken, this can interpose two "commands" -- like a
; delimiter -- but I've never seen this form in the wild
(_ [(command) (list) (pipeline)] . "&" @prepend_space)

; Space between command line arguments
(command argument: _* @append_space @prepend_space)

;; Operators

; Ensure the negation operator is surrounded by spaces
; NOTE This is a syntactic requirement
(negated_command . "!" @prepend_space @append_space)

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
(else_clause . "else" @append_hardline @append_indent_start)

; Keep the "if"/"elif" and the "then" on the same line,
; inserting a delimiter when necessary
(_
  ";"* @do_nothing
  .
  "then" @prepend_delimiter

  (#delimiter! ";")
)

; Finish indent block at "fi", "else" or "elif"
(if_statement
  [
    "fi"
    (else_clause)
    (elif_clause)
  ] @prepend_indent_end @prepend_hardline @append_hardline
)

;; Test Commands

(test_command . (unary_expression) @prepend_space @append_space)

; FIXME The binary_expression node is not being returned by Tree-Sitter
; in the context of a (test_command); it does work in other contexts
; See https://github.com/tweag/topiary/pull/155#issuecomment-1364143677
(binary_expression
   left: _ @append_space
   right: _ @prepend_space
)

;; Loops

; Start loops on a new line
[
  (for_statement)
  (c_style_for_statement)
  ; <TODO: etc.>
] @prepend_hardline

; Indentation block between the "do" and the "done"
(do_group . "do" @append_hardline @append_indent_start)
(do_group "done" @prepend_indent_end @prepend_hardline .)

; Ensure the word list is delimited by spaces in classic for loops
(for_statement value: _* @prepend_space)

; Ensure the loop condition is pleasantly spaced in C-style for loops
(c_style_for_statement
  initializer: _ @prepend_space
  condition: _ @prepend_space
  update: _ @prepend_space @append_space
)

; Keep the loop construct and the "do" on the same line,
; inserting a delimiter when necessary
(_
  ";"* @do_nothing
  .
  (do_group) @prepend_delimiter

  (#delimiter! ";")
)
