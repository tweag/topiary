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

; Append a space to the following keywords and delimiters
[
  ";"
  "case"
  "declare"
  "do"
  "elif"
  "export"
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
] @append_space

; Prepend a space to intra-statment keywords
[
  "in"
] @prepend_space

;; Comments

; Comments come in two flavours: standalone (i.e., it's the only thing
; on a line, starting at the current indent level); and trailing (i.e.,
; following some other statement on the same line, with a space
; interposed). Bash does not have multi-line comments; they are all
; single-line.
;
; The grammar parses all comments as the (comment) node, which are
; siblings under a common parent.
;
; Formatting Rules:
;
; 1. A comment's contents must not be touched; some (namely the shebang)
;    have a syntactic purpose.
; 2. All comments must end with a new line.
; 3. Comments can be interposed by blank lines, if they exist in the
;    input (i.e., blank lines shouldn't be engineered elsewhere).
; 4. A run of standalone comments (i.e., without anything, including
;    blank lines, interposing) should be kept together.
; 5. Trailing comments should only appear after "units of execution" or
;    variable declarations/assignment. (This is despite it being
;    syntactically valid to put them elsewhere.)

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

; "Command" is an epithet for, broadly speaking, a "unit of execution".
; It is such a pervasive and general concept in Bash that we need to
; take care when considering the context. For example, the condition in
; an if statement or while loop is a command, but we don't want to
; insert a new line in these cases.
;
; In terms of the grammar, the following nodes should be considered
; "commands":
;
; * (command)
;   Simple commands (e.g., binaries, builtins, functions, etc.)
;
; * (list)
;   Command lists (i.e., "commands" sequenced by && and ||)
;
; * (pipeline)
;   Command pipelines (i.e., "commands" sequenced by | and |&)
;
; * (subshell)
;   Subshells (i.e., arbitrary code enclosed within parentheses)
;
; * (compound_statement)
;   Compound statements (i.e., arbitrary code enclosed within
;   curly-parentheses)
;
; * (redirected_statement)
;   IO redirection (NOTE These aren't semantically "units of execution"
;   in their own right, but are treated as such due to how the grammar
;   organises them as parent nodes of such units)
;
; * (variable_assignment)
;   Variable assignment (NOTE These aren't "units of execution" at all,
;   but are treated as such to isolate them from their declaration
;   context; see Variables section, below)

; When a "command" is followed by another "command", it should be
; interposed by a new line (a softline, for the sake of single-line
; compound statements and subshells). This, however, is only true in the
; following contexts:
;
; * Top-level statements
; * Multi-line compound statements and subshells
; * In any branch of a conditional or case statement
; * Loop bodies
; * Multi-line command substitutions

(program
  [(command) (list) (pipeline) (subshell) (compound_statement) (redirected_statement) (variable_assignment)] @append_hardline
  .
  [(command) (list) (pipeline) (subshell) (compound_statement) (redirected_statement) (variable_assignment)]
)

; NOTE Single-line compound statements are a thing; hence the softline
(compound_statement
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_spaced_softline
  .
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]
)

; NOTE Single-line subshells are a thing; hence the softline
(subshell
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_spaced_softline
  .
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]
)

(if_statement
  .
  _
  "then"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_hardline
  .
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]
)

(elif_clause
  .
  _
  "then"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_hardline
  .
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]
)

(else_clause
  .
  "else"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_hardline
  .
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]
)

; NOTE Single-line case branches are a thing; hence the softline
(case_item
  .
  _
  ")"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_spaced_softline
  .
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]
)

(do_group
  .
  "do"
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_hardline
  .
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]
)

; NOTE Single-line command substitutions are a thing; hence the softline
(command_substitution
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @append_empty_softline
  .
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]
)

; NOTE Single-line command substitutions are a thing; hence the softline
(command_substitution
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)] @prepend_empty_softline
  .
  [(command) (list) (pipeline) (compound_statement) (subshell) (redirected_statement) (variable_assignment)]
)

; Surround command list and pipeline delimiters with spaces
; NOTE The context here may be irrelevant -- i.e., these operators
; should always be surrounded by spaces -- but they're kept here,
; separately, in anticipation of line continuation support in multi-line
; contexts.
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

; Finish indent block at "fi", "else" or "elif"
(if_statement
  [
    "fi"
    (else_clause)
    (elif_clause)
  ] @prepend_indent_end @prepend_hardline
)

; Keep the "if"/"elif" and the "then" on the same line,
; inserting a spaced delimiter when necessary
; FIXME Why does the space need to be explicitly inserted?
(_
  ";"* @do_nothing
  .
  "then" @prepend_delimiter @prepend_space

  (#delimiter! ";")
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

;; Case Statements

; Indentation block between the "in" and the "esac"
(case_statement
  .
  "case" . _ .  "in" @append_hardline @append_indent_start
  _
  "esac" @prepend_hardline @prepend_indent_end
  .
) @append_hardline

; New (soft)line after branch, which starts an indentation block up
; until its end
(case_item
  ")" @append_spaced_softline @append_indent_start
) @append_indent_end

; Ensure case branch terminators appear on their own line, in a
; multi-line context; or, at least, push the next case branch on to a
; new line in a single-line context
; NOTE The terminator is optional in the final condition, which is why
; we deal with closing the indent block above
(case_item
  [
    ";;"
    ";;&"
    ";&"
  ] @prepend_empty_softline @append_hardline
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
  _
  "done" @prepend_hardline @prepend_indent_end
  .
) @append_hardline

; Ensure the word list is delimited by spaces in classic for loops
(for_statement
  value: _* @prepend_space
)

; Surround the loop condition with spaces in C-style for loops
(c_style_for_statement
  initializer: _ @prepend_space
  update: _ @append_space
)

; Keep the loop construct and the "do" on the same line,
; inserting a spaced delimiter when necessary
; FIXME Why does the space need to be explicitly inserted?
(_
  ";"* @do_nothing
  .
  (do_group) @prepend_delimiter @prepend_space

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
