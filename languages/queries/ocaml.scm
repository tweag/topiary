; Sometimes we want to indicate that certain parts of our source text should
; not be formated, but taken as is. We use the leaf capture name to inform the
; tool of this.
(string) @leaf

; This needs to come before line break patterns.
(
  "}" @prepend_indent_end
)

(else_clause
  _ @append_indent_end
  .
)

(then_clause
  _ @append_indent_end
  .
)

(_
  [
    "->"
    "="
  ]
  _ @append_indent_end
  .
)

; Append line breaks
[
  (external)
  (type_definition)
] @append_hardline

; Root level definitions should have line breaks
(compilation_unit
  (value_definition) @append_hardline
)

; Allow blank line before
[
  (comment)
  (external)
  (type_definition)
  (value_definition)
] @allow_blank_line_before

; Softlines
[
  (comment)
  ("else")
  (else_clause)
  ("then")
  (";")
  ("->")
  ("{")
] @append_spaced_softline

(
  "," @append_space
  (_)
)

(
  [
    ("in")
    ("with")
    ("=")
  ] @append_spaced_softline
  .
  [
    (application_expression)
    (comment)
    (if_expression)
    (let_expression)
    (record_expression)
    (sequence_expression)
  ]
)

[
  ;(comment)
  (else_clause)
  ("|")
  ("}")
] @prepend_spaced_softline

; Append spaces
[
  ("as")
  ("begin")
  (constructor_path)
  ("done")
  ("end")
  ("external")
  ("false")
  ("if")
  (infix_operator)
  ("let")
  ("match")
  ("mutable")
  ("rec")
  ("true")
  ("type")
  ("when")
  ("while")
  ("with")
  ("=")
  ("|")
  ("||")
  ("<-")
  (":")
  ("}")
] @append_space

; Prepend spaces
[
  ("as")
  ("begin")
  ("done")
  ("else")
  ("end")
  ("external")
  ("false")
  ("if")
  ("in")
  (infix_operator)
  ("let")
  ("match")
  ("mutable")
  (parameter)
  ("rec")
  ("then")
  ("true")
  ("type")
  ("when")
  ("while")
  ("with")
  ("=")
  ("||")
  ("->")
  ("<-")
  (":")
  ("{")
] @prepend_space_unless_first_on_line

(application_expression
  (_) @append_space
  (_)
)

; This needs to come after line break patterns.
[
  "else"
  "then"
  "->"
  "{"
] @append_indent_start

[
  "="
] @prepend_indent_start
