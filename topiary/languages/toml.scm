; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
(string) @leaf

; Allow blank line before
[
  (comment)
  (table)
  (table_array_element)
  (pair)
] @allow_blank_line_before

; Append line breaks
[
  (comment)
] @append_hardline

(table
  (pair) @append_hardline
)

(table_array_element
  (pair) @append_hardline
)

(table
  [(bare_key) (dotted_key)]
  .
  "]" @append_hardline
)

(table_array_element
  [(bare_key) (dotted_key)]
  .
  "]]" @append_hardline
)

; Surround spaces
[
  "="
  "{"
  "}"
] @prepend_space @append_space

; Input softlines before all comments. This means that the input decides if a
; comment should have line breaks in front of it.
(comment) @prepend_input_softline

; Softlines. These become either a space or a newline, depending on whether we
; format their node as single-line or multi-line.
[
  ","
] @append_spaced_softline

; Indent arrays. They will only be indented inmulti-line blocks.

(array
  "[" @append_spaced_softline @append_indent_start
)

(array
  "]" @prepend_spaced_softline @prepend_indent_end
)
