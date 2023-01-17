; Configuration
(#language! bash)

; Don't modify string literals
(string) @leaf

; Allow blank line before
; FIXME Blank line spacing around major syntactic blocks is not correct.
[
  (c_style_for_statement)
  (case_item)
  (case_statement)
  (command)
  (comment)
  (compound_statement)
  (for_statement)
  (if_statement)
  (list)
  (pipeline)
  (subshell)
  (while_statement)
  ; TODO: etc.
] @allow_blank_line_before

; Surround with spaces
[
  "case"
  "do"
  "done"
  "elif"
  "else"
  "esac"
  "fi"
  "for"
  "if"
  "in"
  "select"
  "then"
  "until"
  "while"
  (string)
  ; TODO: etc.
] @append_space @prepend_space

;; Compound Statements and Subshells

; Compound statements and subshells are formatted in exactly the same
; way. In a multi-line context, their opening parenthesis triggers a new
; line and the start of an indent block; the closing parenthesis
; finishes that block. In a single-line context, spacing is used instead
; of newlines (NOTE that this is a syntactic requirement of compound
; statements, but not of subshells).
;
; NOTE Despite being isomorphic, the queries for compound statements and
; subshells are _not_ generalised, to ensure parentheses balance.

(compound_statement
  .
  "{" @append_spaced_softline @append_indent_start
)

(compound_statement
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

(subshell
  .
  "(" @append_spaced_softline @append_indent_start
)

(subshell
  ")" @prepend_spaced_softline @prepend_indent_end
  .
)

;; Commands

; NOTE "Command" is an epithet for a "unit of execution":
; * Simple commands (e.g., binaries, builtins, functions, etc.)
; * Command lists
; * Command pipelines
; * Compound statements
; * Subshells
;
; That is, per the grammar:
;   [(command) (list) (pipeline) (compound_statement) (subshell)]

; FIXME I don't think it's possible to insert the necessary line
; continuations; or, at least, it's not possible to insert them only in
; a multi-line context. As such, all multi-line commands are forcibly
; collapsed on to a single line for now... See Issue #172

; One command per line in the following contexts:
; * Top-level
; * Multi-line compound statements and subshells
; * In any branch of a conditional
; * In any branch of a switch statement
; * Within loops
; * <TODO: etc.>
;
; NOTE Because "command" is such a pervasive and general concept, each
; context needs to be individually enumerated to account for exceptions;
; the primary of which being the condition in if statements.
(program
  [(command) (list) (pipeline) (compound_statement) (subshell)] @prepend_hardline
)

; NOTE Single-line compound statements are a thing; hence the softline
(compound_statement
  [(command) (list) (pipeline) (compound_statement) (subshell)] @prepend_spaced_softline
)

; NOTE Single-line subshells are a thing; hence the softline
(subshell
  [(command) (list) (pipeline) (compound_statement) (subshell)] @prepend_spaced_softline
)

(if_statement
  .
  _
  "then"
  [(command) (list) (pipeline) (compound_statement) (subshell)] @prepend_hardline
)

(elif_clause
  .
  _
  "then"
  [(command) (list) (pipeline) (compound_statement) (subshell)] @prepend_hardline
)

(else_clause
  .
  "else"
  [(command) (list) (pipeline) (compound_statement) (subshell)] @prepend_hardline
)

; NOTE Single-line switch branches are a thing; hence the softline
(case_item
  .
  _
  ")"
  [(command) (list) (pipeline) (compound_statement) (subshell)] @prepend_spaced_softline
)

(do_group
  .
  "do"
  [(command) (list) (pipeline) (compound_statement) (subshell)] @prepend_hardline
)

; Surround command list and pipeline delimiters with spaces
; TODO These rules can be subsumed into the list of symbols that are
; surrounded by spaces, above; the context is irrelevant.
; (See https://github.com/tweag/topiary/pull/173#discussion_r1071123588)
(list
  [
    "&&"
    "||"
  ] @append_space @prepend_space
)

(pipeline
  [
    "|"
    "|&"
  ] @append_space @prepend_space
)

; Prepend the asynchronous operator with a space
; NOTE If I'm not mistaken, this can interpose two "commands" -- like a
; delimiter -- but I've never seen this form in the wild
(_
  [(command) (list) (pipeline) (compound_statement) (subshell)]
  .
  "&" @prepend_space
)

; Space between command line arguments
; NOTE If we treat (command) as a leaf node, then commands are formatted
; as is and the below will be ignored. On balance, I think keeping this
; rule, rather than deferring to the input, is the better choice
; (although it's not without its problems; e.g., see Issue #172).
(command
  argument: _* @prepend_space
)

;; Operators

; Ensure the negation operator is surrounded by spaces
; NOTE This is a syntactic requirement
(negated_command
  .
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
(else_clause
  .
  "else" @append_hardline @append_indent_start
)

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
  ] @prepend_indent_end @prepend_hardline
)

;; Test Commands

(test_command
  .
  (unary_expression) @prepend_space @append_space
)

; FIXME The binary_expression node is not being returned by Tree-Sitter
; in the context of a (test_command); it does work in other contexts
; See https://github.com/tweag/topiary/pull/155#issuecomment-1364143677
(binary_expression
   left: _ @append_space
   right: _ @prepend_space
)

;; Switch Statements

; Start switch on a new line
(case_statement) @prepend_hardline

; New line after "in" and start indent block
(case_statement
  .
  _
  "in" @append_hardline @append_indent_start
)

; New (soft)line after switch branch and start indent block
(case_item
  ")" @append_spaced_softline @append_indent_start
)

; Ensure switch branch terminators appear on their own line, in a
; multi-line context; or, at least, push the next switch branch on to a
; new line in a single-line context
; NOTE The terminator is optional in the final condition
(case_item
  [
    ";;"
    ";;&"
    ";&"
  ] @prepend_empty_softline @append_hardline
)

; Finish indent blocks after switch branches and at the "esac", which
; should appear on its own line
(case_item) @append_indent_end
(case_statement
  "esac" @prepend_hardline @prepend_indent_end
  .
)

;; Loops

; Start loops on a new line
[
  (c_style_for_statement)
  (for_statement)
  (while_statement)
] @prepend_hardline

; Indentation block between the "do" and the "done"
(do_group
  .
  "do" @append_hardline @append_indent_start
)

(do_group
  "done" @prepend_indent_end @prepend_hardline
  .
)

; Ensure the word list is delimited by spaces in classic for loops
(for_statement
  value: _* @prepend_space
)

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
