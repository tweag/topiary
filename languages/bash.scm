; Configuration
(#language! bash)

; NOTE There is (currently) no support for line continuations. As such,
; any which are encountered by Topiary will be forcibly collapsed on to
; a single line. (See Issue #172)

; Don't modify string literals, heredocs, comments, atomic "words" or
; variable expansions (simple or otherwise)
; FIXME The first line of heredocs are affected by the indent level
[
  (comment)
  (expansion)
  (heredoc_body)
  (simple_expansion)
  (string)
  (word)
] @leaf

;; Spacing

; Allow blank line before
[
  (c_style_for_statement)
  (case_item)
  (case_statement)
  (command)
  (comment)
  (compound_statement)
  (declaration_command)
  (for_statement)
  (function_definition)
  (if_statement)
  (list)
  (pipeline)
  (redirected_statement)
  (subshell)
  (variable_assignment)
  (while_statement)
] @allow_blank_line_before

; Insert a new line after multi-line syntactic blocks or, for where
; single-line variants exists, after the "closing" subnodes (the
; specificity is to avoid targeting the single-line context)
; FIXME ; [
; FIXME ;   (if_statement)
; FIXME ;   (case_statement)
; FIXME ;   (do_group)
; FIXME ; ] @append_hardline
; FIXME ;
; FIXME ; (subshell
; FIXME ;   ")" @append_empty_softline
; FIXME ;   .
; FIXME ; )
; FIXME ;
; FIXME ; (compound_statement
; FIXME ;   "}" @append_empty_softline
; FIXME ;   .
; FIXME ; )

; A run of "units of execution" (see below, sans variables which are
; special) and function definitions should be followed by a new line,
; before a multi-line syntactic block or variable.
; FIXME ; (
; FIXME ;   [
; FIXME ;     (command)
; FIXME ;     (compound_statement)
; FIXME ;     (function_definition)
; FIXME ;     (list)
; FIXME ;     (pipeline)
; FIXME ;     (redirected_statement)
; FIXME ;     (subshell)
; FIXME ;   ] @append_empty_softline
; FIXME ;   .
; FIXME ;   [
; FIXME ;     (c_style_for_statement)
; FIXME ;     (case_statement)
; FIXME ;     (compound_statement)
; FIXME ;     (declaration_command)
; FIXME ;     (for_statement)
; FIXME ;     (function_definition)
; FIXME ;     (if_statement)
; FIXME ;     (subshell)
; FIXME ;     (variable_assignment)
; FIXME ;     (while_statement)
; FIXME ;   ]
; FIXME ; )

; A run of variable declarations and assignments should be followed by a
; new line, before anything else
; FIXME ; (
; FIXME ;   [
; FIXME ;     (declaration_command)
; FIXME ;     (variable_assignment)
; FIXME ;   ] @append_hardline
; FIXME ;   .
; FIXME ;   [
; FIXME ;     (c_style_for_statement)
; FIXME ;     (case_statement)
; FIXME ;     (command)
; FIXME ;     (compound_statement)
; FIXME ;     (compound_statement)
; FIXME ;     (for_statement)
; FIXME ;     (function_definition)
; FIXME ;     (if_statement)
; FIXME ;     (list)
; FIXME ;     (pipeline)
; FIXME ;     (redirected_statement)
; FIXME ;     (subshell)
; FIXME ;     (subshell)
; FIXME ;     (while_statement)
; FIXME ;   ]
; FIXME ; )

; Surround keywords with spaces
[
  "case"
  "declare"
  "do"
  "done"
  "elif"
  "else"
  "esac"
  "export"
  "fi"
  "for"
  "if"
  "in"
  "local"
  "readonly"
  "select"
  "then"
  "typeset"
  "until"
  "while"
] @append_space @prepend_space

;; Comments

; FIXME
(comment) @append_hardline

;; Compound Statements and Subshells

; Compound statements and subshells are formatted in exactly the same
; way. In a multi-line context, their opening parenthesis triggers a new
; line and the start of an indent block; the closing parenthesis
; finishes that block. In a single-line context, spacing is used instead
; of new lines (NOTE that this is a syntactic requirement of compound
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
; * Redirection statements (NOTE these aren't "units of execution" in
;   their own right, but are treated as such due to how the grammar
;   organises them as parent nodes of such units)
; * Variable assignment (NOTE these aren't "units of execution" at all,
;   but are treated as such to isolate them from their declaration
;   context; see Variables section, below)
;
; That is, per the grammar:
;   [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]

; One command per line in the following contexts:
; * Top-level
; * Multi-line compound statements and subshells
; * In any branch of a conditional
; * In any branch of a switch statement
; * Within loops
; * Multi-line command substitutions
;
; NOTE Because "command" is such a pervasive and general concept, each
; context needs to be individually enumerated to account for exceptions;
; the primary of which being the condition in if statements.
(program
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_hardline
)

; NOTE Single-line compound statements are a thing; hence the softline
(compound_statement
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_spaced_softline
)

; NOTE Single-line subshells are a thing; hence the softline
(subshell
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_spaced_softline
)

(if_statement
  .
  _
  "then"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_hardline
)

(elif_clause
  .
  _
  "then"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_hardline
)

(else_clause
  .
  "else"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_hardline
)

; NOTE Single-line switch branches are a thing; hence the softline
(case_item
  .
  _
  ")"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_spaced_softline
)

(do_group
  .
  "do"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_hardline
)

; NOTE Single-line command substitutions are a thing; hence the softline
(command_substitution
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_empty_softline
)

; NOTE Single-line command substitutions are a thing; hence the softline
(command_substitution
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @prepend_empty_softline
)

; Surround command list and pipeline delimiters with spaces
; NOTE These queries could be subsumed into the list of symbols that are
; surrounded by spaces (above), as the context is irrelevant. However,
; they're kept here, separately, in anticipation of line continuation
; support in multi-line contexts.
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
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement)]
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

; Ensure the negation operator is surrounded by spaces
; NOTE This is a syntactic requirement
(negated_command
  .
  "!" @prepend_space @append_space
)

; Multi-line command substitutions become an indent block
(command_substitution
  .
  (_) @prepend_empty_softline @prepend_indent_start
)

(command_substitution
  ")" @prepend_empty_softline @prepend_indent_end
  .
)

;; Redirections

; Insert a space before all redirections, but _not_ after the operator
(redirected_statement
  redirect: _* @prepend_space
)

; ...with the exceptions of herestrings, that are spaced
(herestring_redirect (_) @prepend_space)

; Ensure heredocs start on a new line, after their start marker, and
; there is a new line after their end marker
; NOTE This is a syntactic requirement
(heredoc_start) @append_hardline
(heredoc_body) @append_hardline

;; Conditionals

; New line after conditionals
[
  (if_statement)
  (elif_clause)
  (else_clause)
] @append_hardline

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
  (unary_expression
    _ @prepend_space
  ) @append_space
)

; FIXME The binary_expression node is not being returned by Tree-Sitter
; in the context of a (test_command); it does work in other contexts
; See https://github.com/tweag/topiary/pull/155#issuecomment-1364143677
(binary_expression
   left: _ @append_space
   right: _ @prepend_space
)

;; Switch Statements

; New line after switch statement
(case_statement) @append_hardline

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

; Finish indent blocks after switch branches and at the "esac"
(case_item) @append_indent_end
(case_statement
  "esac" @prepend_indent_end
  .
)

;; Loops

; Start loops on a new line
; FIXME We may not need this any more
; FIXME ; [
; FIXME ;   (c_style_for_statement)
; FIXME ;   (for_statement)
; FIXME ;   (while_statement)
; FIXME ; ] @prepend_hardline

; Indentation block between the "do" and the "done"
(do_group
  .
  "do" @append_hardline @append_indent_start
)

(do_group
  "done" @prepend_indent_end @append_hardline
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

;; Function Definitions

; NOTE Much of the formatting work for function definitions is done by
; whatever already-defined queries apply to the function body (e.g.,
; (compound_statement), etc.). All we do here is ensure functions get
; their own line and put a space between its name and the body.
; FIXME We may not need this any more
; FIXME ; (function_definition) @prepend_hardline

(function_definition
  body: _ @prepend_space
)

; NOTE The "function" keyword in function definitions is optional and
; thus usually considered redundant. Therefore we delete it, if it's
; present in the input.
(function_definition
  .
  "function" @delete
)

;; Variable Declaration, Assignment and Expansion

; NOTE Assignment only gets a new line when not part of a declaration;
; that is, all the contexts in which units of execution can appear.
; Hence the queries for this are defined above. (My kingdom for a
; negative anchor!)

; Declaration on a new line
; FIXME We may not need this any more
; FIXME ; (declaration_command) @prepend_hardline

; Multiple variables can be exported (and assigned) at once
(declaration_command
  .
  "export"
  [(variable_name) (variable_assignment)] @prepend_space
)

; Environment variables assigned to commands inline need to be spaced
(command
  (variable_assignment) @append_space
)

; NOTE The (simple_expansion), for `$foo`, and (expansion), for `${foo}`
; and friends, node types exist. We consider them as leaves (see above).
; However, it would be _really_ nice if we could write a query that
; converts all (simple_expansions) into (expansions). It can almost be
; done with delimiters, but it doesn't quite work :( For example:
;
; (simple_expansion (variable_name) @prepend_delimiter (#delimiter! "{"))
; (simple_expansion (variable_name) @append_delimiter (#delimiter! "}"))
;
; See https://github.com/tweag/topiary/pull/179#discussion_r1073202151
