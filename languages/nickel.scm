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

; Don't insert spaces between the negation operator and its operand
(infix_expr
  .
  "-" @append_antispace
  .
  (infix_expr)
  .
)

;; Comments

(comment) @prepend_input_softline @append_hardline

;; Symbol Definitions
; i.e., Let bindings and record fields

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

; Start a let binding's RHS on a new line, in a multi-line context.
(let_in_block
  (#scope_id! "let_term")
  "=" @begin_scope
  .
  (term) @end_scope
)

(let_in_block
  (#scope_id! "let_term")
  (term) @prepend_spaced_scoped_softline
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

;; Container Types
; Arrays, records, dictionaries and enums

; We don't want to add spaces/newlines in empty records, so the
; following query only matches if a named node exists within the record
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

(let_expr
  (let_in_block) @append_spaced_softline
)

(let_in_block
  "=" @append_indent_start
  .
  t1: (_) @append_indent_end @append_spaced_softline
)

(match_expr
  "{" @append_spaced_softline @append_indent_start
  "}" @prepend_indent_end @prepend_spaced_softline @append_spaced_softline
)

(match_expr
  "," @append_spaced_softline
)

(ite_expr
  "then" @prepend_spaced_softline @append_spaced_softline @append_indent_start
  t1: (term) @append_spaced_softline @append_indent_end
  "else" @append_indent_start @append_spaced_softline
  t2: (term) @append_indent_end
)

(infix_b_op_6
  "&"
) @prepend_spaced_softline

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

(destruct
  "{" @append_spaced_softline @append_indent_start
  "}" @prepend_indent_end @prepend_spaced_softline
)

(destruct
  "," @append_spaced_softline
)

(match_case
  "=>" @append_spaced_softline @append_indent_start
  (_) @append_indent_end
  .
)
