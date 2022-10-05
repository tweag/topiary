; Configuration
(#language! ocaml)

; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
(
  [
    (character)
    (string)
  ]
) @leaf

; Allow blank line before
[
  (comment)
  (external)
  (type_definition)
  (value_definition)
] @allow_blank_line_before

; Append line breaks
[
  (external)
  (type_definition)
] @append_hardline

; Consecutive definitions must be separated by line breaks
(
  (value_definition) @append_hardline
  .
  (value_definition)
)

; Surround spaces
[
  "as"
  "assert"
  "begin"
  "do"
  "else"
  "external"
  "false"
  (infix_operator)
  "let"
  "match"
  "mutable"
  (parameter)
  "rec"
  "then"
  "true"
  "type"
  "when"
  "while"
  "with"
  "="
  "|"
  "||"
  "->"
  "<-"
  ":"
  "{"
  "}"
] @prepend_space @append_space

; Prepend spaces
[
  "done"
] @prepend_space

; Append spaces
[
  "if"
  ";"
] @append_space

; Put a space after commas, except the last one.
(
  "," @append_space
  .
  (_)
)

; Put a space after arguments, except the last one.
(application_expression
  (_) @append_space
  .
  (_)
)

; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. But don't put a
; softline directly in front of commas or semicolons.

(comment) @prepend_input_softline

(
  (comment) @append_input_softline
  .
  [ "," ";" ]* @do_nothing
)

; Softlines. These become either a space or a newline, depending on whether we
; format their node as single-line or multi-line. If there is a comment
; following, we don't add anything, because the input softlines and spaces above
; will already have sorted out the formatting.
(
  [
    "begin"
    "else"
    "then"
    "->"
    "{"
    ";"
  ] @append_spaced_softline
  .
  (comment)* @do_nothing
)

; Always put softlines before these:
[
  "end"
  (else_clause)
  (infix_operator)
  "|"
  "}"
] @prepend_spaced_softline

; Multi-line definitions must have a linebreak before "in":
;
; let a =
;   expression
;   in
;   expression
(
  (value_definition
    (_) @append_spaced_softline
    .
  )
  .
  "in"
)

; The following are many constructs that need a softline.
(
  [
    "in"
    "with"
    "="
  ] @append_spaced_softline
  .
  [
    (application_expression)
    (if_expression)
    (let_expression)
    (product_expression)
    (record_expression)
    (sequence_expression)
    (value_path)
  ]
)

; Put a semicolon delimiter after field declarations, unless they already have
; one, in which case we do nothing. 
(
  (field_declaration) @append_delimiter
  .
  ";"* @do_nothing
  (#delimiter! ";")
)

; Indenting. This will only do anything in multi-line blocks. In single-line
; blocks they do nothing.

; Start an indented block after these
[
  "begin"
  "else"
  "then"
  "->"
  "{"
] @append_indent_start

; End the indented block before these
[
  "end"
  "}"
] @prepend_indent_end

; End the indented block after these
[
  (else_clause)
  (then_clause)
] @append_indent_end

(_
  [
    "->"
  ]
  _ @append_indent_end
  .
)

; Make an indented block after "=" in let bindings
(let_binding
  "=" @append_indent_start
  _ @append_indent_end
  .
)

; Make an indented block after the first term in a long if expression
;
; if len < 0
;  || srcoff < 0
;  || srcoff > src.position - len then
;
(if_expression
  (infix_expression
    .
    (_) @append_indent_start
    (_) @append_indent_end
    .
  )
)
