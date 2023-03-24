; Configuration
(#language! nickel)

;; General Spacing

; The following nodes in our source text should not be formatted
[
  (static_string)
  (str_chunks_single)
  (str_chunks_multi)
  (builtin)
] @leaf

; Allow a blank line before the following nodes
[
  (comment)
  (record_field)
  (record_last_field)
] @allow_blank_line_before

; Surround with spaces: keywords, operators, annotation markers
(
  [
    "if"
    "then"
    "else"
    "forall"
    "in"
    "let"
    "match"
    "null"
    "true"
    "false"
    "fun"
    "import"
    "default"
    "doc"
    "force"
    "optional"
    "priority"
    "|"
    ":"
    "?"
    "="
    "rec"
    "Array"
    "Dyn"
    "Num"
    "Bool"
    "Str"
    "->"
    "=>"
    (interpolation_start)
    (interpolation_end)
    ; Infix operators
    "++"
    "@"
    "*"
    "/"
    "%"
    "+"
    "-"
    "!"
    "&"
    "|>"
    "<"
    "<="
    ">"
    ">="
    "=="
    "!="
    "&&"
    "||"
  ] @prepend_space @append_space
)

; Don't insert spaces before the following delimiters
; NOTE This will destroy the space in a polymorphic record tail. For
; example: forall. { x: Number; a } -> {; a}
[
  ","
  ";"
  "."
] @prepend_antispace

; Don't insert spaces immediately inside parentheses
"(" @append_antispace
")" @prepend_antispace

; Don't insert spaces between infix operators and their operand
(infix_expr
  .
  [
    "-"
    (infix_u_op_5 "!")
  ] @append_antispace
  .
  (infix_expr)
  .
)

; Flow a chain of infix expressions over new lines, in a multi-line
; context. Note that we _don't_ want this to happen for comparison
; operators, which fall under nodes (infix_b_op_7) and (infix_b_op_8).
(uni_term
  (#scope_id! "infix_chain")
  (infix_expr) @begin_scope
) @end_scope

(infix_expr
  (#scope_id! "infix_chain")
  (infix_expr)
  .
  [
    (infix_b_op_2) ; ++ and @
    (infix_b_op_3) ; *, / and %
    (infix_b_op_4) ; + and -
    (infix_b_op_6) ; & and |>
    (infix_lazy_b_op_9) ; &&
    (infix_lazy_b_op_10) ; ||
  ] @prepend_spaced_scoped_softline
  .
  (infix_expr)
)

; Surround all polymorphic type variables with spaces
(forall
  (ident) @append_space @prepend_space
)

;; Comments

(comment) @prepend_input_softline @append_hardline

;; Symbol Definitions
; i.e., Let expressions and record fields

; A let expression looks like:
;
;   let [rec] IDENT = EXPR in EXPR
;
; The binding expression should appear on a new line, indented, if its
; RHS is multi-line (pushing the "in" to an unindented new line).
; Similarly, the result expression (i.e., after the "in") should appear
; on an indented new line, if that is multi-line.

(let_in_block
  (#scope_id! "let_binding_rhs")
  "=" @begin_scope
  .
  (term)
  .
  "in" @end_scope
)

(let_in_block
  (#scope_id! "let_binding_rhs")
  (term) @prepend_spaced_scoped_softline @prepend_indent_start
  "in" @prepend_indent_end @prepend_spaced_scoped_softline
)

(let_expr
  (#scope_id! "let_result")
  (let_in_block
    "in" @begin_scope
    .
  )
  .
  (term) @end_scope
)

(let_expr
  (#scope_id! "let_result")
  (term) @prepend_spaced_scoped_softline @prepend_indent_start @append_indent_end
)

;; Annotations

; Create a scope that covers at least all annotation atoms; that is,
; children of the (annot) node. When an assignment is also involved, we
; have another scope that extents to also cover the equals sign.
(
  (#scope_id! "signature")
  _ @begin_scope
  .
  (annot) @end_scope
)

(
  (#scope_id! "assignee")
  (_) @begin_scope
  .
  (annot)*
  .
  "=" @end_scope
)

; Start an indentation block from the start of the annotations to the
; end of the enclosing node
(_
  (annot) @prepend_indent_start
) @append_indent_end

; Put each annotation and the equals sign on a new line, in a multi-line
; context.
(
  (#scope_id! "signature")
  (annot_atom) @prepend_spaced_scoped_softline
)

; FIXME This breaks idempotency for multi-line let expressions with two
; or more annotations on the same line as the equals sign! For example:
;
;  let x
;    : TYPE | ANNOT = 1 in x
;
; It also doesn't do the right thing (push the equals sign to a new
; line) if it lives on the same line as the last annotation in the
; input. For example:
;
;   let x
;     : TYPE = 1 in x
;
; These forms are not attested in the Nickel standard library, as of
; writing.
(_
  (#scope_id! "assignee")
  "=" @prepend_spaced_scoped_softline
)

; Break a multi-line polymorphic type annotation after the type
; variables, starting an indentation block
(forall
  "." @append_spaced_softline @append_indent_start
) @append_indent_end

;; Functions

; Start a function's definition on a new line, in a multi-line context.
; This also defines an indentation block.
(fun_expr
  (#scope_id! "function_definition")
  "=>" @begin_scope @append_indent_start
) @append_indent_end @end_scope

(fun_expr
  (#scope_id! "function_definition")
  (term) @prepend_spaced_scoped_softline
)

(fun_expr
  (pattern) @append_space
)

; Application operator is space, so we put it between identifiers.
(applicative
  t1: (applicative) @append_space
)

;; Conditionals

; Flow multi-line match cases into an indented block after the =>
(match_case
  "=>" @append_spaced_softline @append_indent_start
) @append_indent_end

; if...then...else expressions can either be single or multi-line. In a
; multi-line context, they will be formatted like so:
;
;  if CONDITION then
;    TRUE_TERM
;  else
;    FALSE_TERM
;
; This style has precedent from the manually formatted stdlib. (An
; alternative style is to give the "then" token its own line.)
(ite_expr
  "then" @append_spaced_softline @append_indent_start
  t1: (term) @append_indent_end
  "else" @prepend_spaced_softline @append_spaced_softline @append_indent_start
  t2: (term) @append_indent_end
)

;; Container Types
; i.e., Arrays, records (and dictionaries, vicariously) and enums

; We don't want to add spaces/new lines in empty records, so the
; following query only matches if a named node exists within the record
; NOTE This rule also applies to (match) and (destruct) patterns
(_
  (#scope_id! "container")
  .
  "{" @append_spaced_softline @append_indent_start @begin_scope
  (_)
  "}" @prepend_indent_end @prepend_spaced_softline @end_scope
  .
)

; As with records, we don't want internal spacing to apply if the array
; is empty; this restriction also applies to arrays of exactly one
; element, which is usually short (e.g., [x]).
(_
  (#scope_id! "container")
  .
  "[" @append_spaced_softline @append_indent_start @begin_scope
  (_)
  (_)
  "]" @prepend_indent_end @prepend_spaced_softline @end_scope
  .
)

; It doesn't really make sense for an enum to
; have no members, so we ignore that case
(_
  (#scope_id! "container")
  .
  "[|" @append_spaced_softline @append_indent_start @begin_scope
  "|]" @prepend_indent_end @prepend_spaced_softline @end_scope
  .
)

(
  (#scope_id! "container")
  [
    ","
    ";"
  ] @append_spaced_scoped_softline
)
