; Configuration
(#language! nickel)

; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
[
  (static_string)
  (str_chunks_single)
  (str_chunks_multi)
  (builtin)
] @leaf

; Allow blank line before
[
  (comment)
  (record_field)
  (record_last_field)
] @allow_blank_line_before

(comment) @prepend_input_softline

(
  (comment) @append_input_softline
  .
  ["," ";"]* @do_nothing
)

; Surround spaces
; A space is put after, and before keywords.
; It is also put before and after "|", ":" and "?" separating annotation from the annotated object.
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
    ;  We put spaces around infix operators
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

(let_expr
  (let_in_block) @append_spaced_softline
)

(let_in_block
  "=" @append_spaced_softline @append_indent_start
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
