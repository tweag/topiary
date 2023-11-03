(symbol) @leaf

[
  (symbol)
] @append_space @prepend_space

[
  (list)
] @allow_blank_line_before

(expression) @append_empty_softline

(list
  .
  "(" @append_antispace
  .
  (expression) @append_empty_softline @append_indent_start

  ")" @prepend_antispace @prepend_empty_softline @prepend_indent_end
  .
)
