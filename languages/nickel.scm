; Configuration
(#language! nickel)

;; General

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
    "{"
    "}"
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

;; TIDY FROM HERE ;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;;

(let_expr
  (let_in_block) @append_spaced_softline
)

(let_in_block
  "=" @append_input_softline @append_indent_start
  .
  t1: (_) @append_indent_end @append_spaced_softline
)

(fun_expr
  "=>" @append_spaced_softline @append_indent_start
  (_) @append_indent_end
  .
)

(fun_expr
  (pattern) @append_space
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

(
  (infix_b_op_6
    "&"
  ) @prepend_spaced_softline
)

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

(uni_record
  "{" @append_spaced_softline @append_indent_start
  "}" @prepend_indent_end @prepend_spaced_softline
)

(uni_record
  "," @append_spaced_softline
)

(uni_record
  ";" @append_spaced_softline
)

; We want the same rule for arrays as for records,
; but array does not have a specific syntactic class, so we use the one of atoms.
(atom
  .
  "[" @append_spaced_softline @append_indent_start
  (term)
  "]" @prepend_indent_end @prepend_spaced_softline
  .
)

(atom
  "," @append_spaced_softline
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

(type_atom
  [
    "[|"
    "{"
  ] @append_spaced_softline @append_indent_start
  [
    "|]"
    "}"
  ] @prepend_indent_end @prepend_spaced_softline
)

(type_atom
  [
    ","
    ";"
  ] @append_spaced_softline
)

; Application operator is space, so we put it between identifiers.
(applicative
  t1: (applicative) @append_space
)
