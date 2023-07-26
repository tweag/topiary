; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
[
  (capture)
  (identifier)
  (string)
] @leaf

(comment) @prepend_input_softline @append_hardline @allow_blank_line_before

; Elements at top-level must be alone on their line. Blank lines are allowed
(program
  _ @allow_blank_line_before @prepend_hardline
)

; There is a large category of nodes that must be separated by spaces
(
  [
    (anonymous_node)
    (capture)
    (identifier)
    (grouping)
    (named_node)
    (parameters)
    (quantifier)
    ":"
    "."
    ")"
    "]"
  ] @append_space
  .
  [")" "]" ":"]? @do_nothing
  .
  [
    (anonymous_node)
    (capture)
    (grouping)
    (list)
    (named_node)
    "."
    "("
    "["
    "_"
  ]
)
(anonymous_node
  (capture) @prepend_space
)

; The grammar always includes a (parameters) child node under
; (predicate), even when there are none. Topiary will deal with the
; zero-byte nodes, so we just need to set the spacing correctly.
(predicate
  (parameters
    _ @prepend_space
  )
)

; Line breaks and indentation in multi-line lists and nodes
(list
  "[" @append_indent_start @append_empty_softline
  "]" @prepend_indent_end @prepend_empty_softline
)

(grouping
  "(" @append_indent_start @append_empty_softline
  ")" @prepend_indent_end @prepend_empty_softline
)

(named_node
  "("
  .
  [
    (identifier)
    "_"
  ] @append_indent_start @append_empty_softline
  ")" @prepend_indent_end @prepend_empty_softline
)

(
  [
    (anonymous_node)
    (field_definition)
    (list)
    (named_node)
    (predicate)
    "."
  ] @append_spaced_softline
  .
  [
    (anonymous_node)
    (field_definition)
    (list)
    (named_node)
    (predicate)
    "."
  ]
)
