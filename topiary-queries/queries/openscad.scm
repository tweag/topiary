; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
[
  (block_comment)
  (line_comment)
  (string)
] @leaf

; Allow blank line before
[
  (use_statement)
  (include_statement)
  (block_comment)
  (line_comment)
  (function_item)
  (transform_chain)
  (module_item)
  (expression)
] @allow_blank_line_before

; Keywords

; Surround spaces
[
  "module"
  "function"
  "let"
  "include"
  "assign"
  "use"
  "each"
  "||"
  "&&"
  "=="
  "!="
  "<"
  ">"
  "<="
  ">="
  "+"
  "-"
  "*"
  "/"
  "%"
  "^"
  "!"
  "="
  "?"
  ":"
] @prepend_space @append_space

; Colon should have whitespace trimmed in a range delimiter
(range ":" @prepend_antispace @append_antispace)

; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. A line comment
; always ends with a line break.
[
  (block_comment)
  (line_comment)
] @prepend_input_softline

; Append line breaks. If there is a comment following, we don't add anything,
; because the input softlines and spaces above will already have sorted out the
; formatting.
(
  [
    (var_declaration)
    (use_statement)
    (include_statement)
    (function_item)
    (module_item)
    (transform_chain)
  ] @append_spaced_softline
  .
  [
    (block_comment)
    (line_comment)
  ]* @do_nothing
)

(line_comment) @append_hardline

(block_comment) @multi_line_indent_all

; Allow line break after block comments
(
  (block_comment)
  .
  _ @prepend_input_softline
)

; indent the body of a function
(function_item
  (parameters)
  .
  "=" @append_spaced_softline @append_indent_start
  (expression)
  ";" @prepend_indent_end
)

; Append softlines, unless followed by comments.
(
  [
    ","
    ";"
  ] @append_spaced_softline
  .
  [(block_comment) (line_comment)]* @do_nothing
)

; Never put a space before a comma
(
  "," @prepend_antispace
)

; Don't insert spaces between the operator and their expression operand
(unary_expression
  _ @append_antispace
  .
  (expression)
)

; Don't insert spaces between the starting '[' and ending ']' of an index expression
(index_expression
  value: _ @append_antispace
  index: _ @prepend_antispace @append_antispace
)
