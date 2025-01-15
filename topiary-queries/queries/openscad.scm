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
  (statement)
  (line_comment)
  (block_comment)
  (function_item)
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
  "else"
  "if"
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
    (function_item)
    (module_item)
    (statement)
  ] @append_spaced_softline
  .
  [
    "else"
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

; Append softlines, unless followed by comments.
(
  [
    ","
    ";"
  ]
  .
  [(block_comment) (line_comment)]* @do_nothing
)

; Never put a space before a comma
(
  "," @prepend_antispace
)

; Don't insert spaces between the operator and their expression operand
; '-x' v.s. '- x'
(unary_expression _ @append_antispace . (expression))

; ================================================================================
; functions & modules
; ================================================================================

; indent the body of a function
(function_item
  (parameters)
  .
  "=" @append_spaced_softline @append_indent_start
  (expression)
  ";" @prepend_indent_end
)

; module calls in a transformation chain will follow each other
; sometimes staying on the same line and sometimes having a linebreak,
; each linebreak typically also starts an indent scope
(transform_chain) @prepend_input_softline
(transform_chain
  (module_call) @append_indent_start
  (transform_chain) @append_indent_end
)
; (function_call
;   "=" @prepend_antispace @append_antispace
; )

; ================================================================================
; blocks & expressions
; ================================================================================
(arguments
  "(" @append_empty_softline @append_indent_start
  "," @append_input_softline
  ")" @prepend_indent_end
)

; differentiate parameter definitions from parameter invocation,
; module/function definitions have param separation while
; module/function calls have none, this space on chained function/module calls
; and provides visual distinction between definitions and calls
(arguments
  (assignment
    "=" @append_antispace @prepend_antispace
  )
)

(union_block
  .
  "{" @append_spaced_softline @append_indent_start @prepend_space
  _
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

;{ ... } else {
; ((union_block) @append_space . "else")

; everything except `union_block` after a for/if/else statement should be a spaced_softline
; but a union
(if_block
  (parenthesized_expression) @append_spaced_softline @append_indent_start
  .
  [
    (for_block)
    (intersection_for_block)
    (if_block)
    (let_block)
    (assign_block)
    (modifier_chain)
    (transform_chain)
    (include_statement)
    (assert_statement)
  ] @append_indent_end @append_spaced_softline
)
(
  "else" @append_spaced_softline @append_indent_start
  .
  [
    (for_block)
    (intersection_for_block)
    (let_block)
    (assign_block)
    (modifier_chain)
    (transform_chain)
    (include_statement)
    (assert_statement)
  ] @append_indent_end
) @prepend_spaced_softline

; scope is triggered by the presence of a (union_block) consequce on the intersection_for_block;
; and extends to the "else if" and "else" portions
(
  "else"? @do_nothing
  (if_block
    (#scope_id! "if_union")
  ) @prepend_begin_scope @append_end_scope
)

(if_block
  (#scope_id! "if_union")
  (_
    .
    "{" @append_spaced_scoped_softline
    "}" @prepend_spaced_scoped_softline
    .
  )
)
