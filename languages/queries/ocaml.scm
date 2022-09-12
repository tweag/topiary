; Sometimes we want to indicate that certain parts of our source text should
; not be formated, but taken as is. We use the leaf capture name to inform the
; tool of this.
(
  [
    (character)
    (string)
  ]
) @leaf

; This needs to come before line break patterns.
(
  [
    "end"
    "}"
  ] @prepend_indent_end
)

(else_clause
  _ @append_indent_end
  .
)

(then_clause
  _ @append_indent_end
  .
)

(let_binding
  ("=")
  _ @append_indent_end
  .
)

(_
  [
    "->"
  ]
  _ @append_indent_end
  .
)

(if_expression
  (infix_expression
    (_) @append_indent_end
    .
  )
)

; Append line breaks
[
  (external)
  (type_definition)
] @append_hardline

; Allow blank line before
[
  (comment)
  (external)
  (type_definition)
  (value_definition)
] @allow_blank_line_before

; Softlines
[
  ("begin")
  (comment)
  ("else")
  ("then")
  (";")
  ("->")
  ("{")
] @append_spaced_softline

(
  (value_definition
    (_) @append_spaced_softline
    .
  )
  .
  "in"
)

(
  (value_definition) @append_hardline
  (value_definition)
)

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
    (product_expression)
    (record_expression)
    (sequence_expression)
    (value_path)
  ]
)

[
  ("end")
  (else_clause)
  (infix_operator)
  ("|")
  ("}")
] @prepend_spaced_softline

; Append spaces
[
  ("as")
  ("assert")
  ("do")
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
  ("do")
  ("done")
  ("else")
  ("external")
  ("false")
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
] @prepend_space

(application_expression
  (_) @append_space
  (_)
)

; This needs to come after line break patterns.
[
  "begin"
  "else"
  "then"
  "->"
  "{"
] @append_indent_start

(let_binding
  "=" @prepend_indent_start
)

(if_expression
  (infix_expression
    .
    (_) @append_indent_start
  )
)
