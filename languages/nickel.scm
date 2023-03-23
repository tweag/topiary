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

; Flow a chain of infix expressions over
; new lines, in a multi-line context
(uni_term
  (#scope_id! "infix_chain")
  (infix_expr) @begin_scope
) @end_scope

(infix_expr
  (#scope_id! "infix_chain")
  (infix_expr)
  .
  (_) @prepend_spaced_scoped_softline
  .
  (infix_expr)
)

;; Comments

(comment) @prepend_input_softline @append_hardline

;; Symbol Definitions
; i.e., Let expressions and record fields

; Create a scope that covers all annotation atoms, if any,
; which are children of the (annot) node, *and* the equal sign. This
; also defines an indentation block.
;
; NOTE This query will only match when annotations are present; thus a
; "bare" signature, with just an equal sign, will not get a softline,
; regardless of context. This behaviour can be changed by quantifying
; the (annot) node with the Kleene star; with the consequence of keeping
; the signature together if it's written on one line (albeit a different
; one to the defined symbol). For example:
;
;   {
;     foo
;       | some | annotations = 1
;   }
;
; The unquantified behaviour is probably a better trade-off, as bare
; signatures are short and so more conducive to a single-line.
;
;   {
;     foo
;       | some
;       | annotations
;       = 1
;   }
(
  (#scope_id! "signature")
  _ @begin_scope
  .
  (annot) @prepend_indent_start
  .
  "=" @append_indent_end @end_scope
)

; Put each annotation and the equals sign on a new line, in a multi-line
; context. Type annotations do not get a new line; this is because they
; can be nested and it's not(?) possible to deduce the depth with
; queries alone. For example:
;
;   {
;     foo : { foo : String, bar : Number } = ...
;   }
;
;   {
;     foo : {
;       foo : String,
;       bar : Number
;     }
;     = ...
;   }
(
  (#scope_id! "signature")
  [
    (annot_atom "|")
    "="
  ] @prepend_spaced_scoped_softline
)

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

; If a record field's value is multi-line, then push it on to an
; indented new line (like let expressions). This has a single "ugly"
; case: when a multi-line value follows multi-line annotations, the
; equals sign will be alone on its own line.
(record_field
  (#scope_id! "record_field_rhs")
  "=" @begin_scope
  .
  (term) @end_scope
)

(record_field
  (#scope_id! "record_field_rhs")
  (term) @prepend_indent_start @prepend_spaced_scoped_softline @append_indent_end
)

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
  "then" @append_spaced_softline
  t1: (term) @prepend_indent_start @append_indent_end
  "else" @prepend_spaced_softline @append_spaced_softline
  t2: (term) @prepend_indent_start @append_indent_end
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

;; TIDY FROM HERE ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(forall
  "." @append_spaced_softline @append_indent_start
  (_) @append_indent_end
  .
)

(forall
  (ident) @append_space
  .
  (ident)
)
