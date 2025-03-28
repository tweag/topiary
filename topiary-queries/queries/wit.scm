; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
[
  (block_comment)
  (line_comment)
] @leaf

(line_comment) @append_hardline
(block_comment) @multi_line_indent_all
; Allow line break after block comments
(
  (block_comment)
  .
  _ @prepend_input_softline
)

; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. A line comment
; always ends with a line break.
[
  (block_comment)
  (line_comment)
] @prepend_input_softline

; Allow blank line before
[
  (interface_item)
  (type_item)
  (enum_items)
  (variant_items)
  (resource_item)
  (record_item)
  (flags_items)
  (type_item)
] @allow_blank_line_before

[
  "use"
  "func"
  "static"
  "async"
  "type"
  "interface"
  "world"
  "package"
  "resource"
  "record"
  "enum"
  "flags"
  "variant"
  "include"
  "import"
  "export"
  "as"
  "with"
  "->"
  "="
] @prepend_space @append_space

; Append line breaks. If there is a comment following, we don't add anything,
; because the input softlines and spaces above will already have sorted out the
; formatting.
(
  [
    (export_item)
    (func_item)
    (import_item)
    (include_item)
    (interface_item)
    (package_decl)
    (resource_item)
    (type_item)
    (use_item)
    (resource_method)
    (since_gate)
    (deprecated_gate)
    (unstable_gate)
  ] @append_spaced_softline @prepend_hardline
  .
  [
    (block_comment)
    (line_comment)
  ]* @do_nothing
)
[
  (since_gate)
  (deprecated_gate)
  (unstable_gate)
] @append_hardline

(body
  .
  "{" @append_hardline @append_indent_start @prepend_space
  _
  "}" @prepend_hardline @prepend_indent_end
  .
)
(use_item
  "{" @append_antispace @append_indent_start @prepend_antispace
  (use_names_list)
  "}" @prepend_antispace @prepend_indent_end
)

(enum_cases "," @append_hardline)
(flags_items (body "," @append_hardline))
(record_item (body "," @append_hardline))
(variant_cases "," @append_hardline)
; ==================
; Delimiters
; ==================
; Never put a space before a comma or semicolon
(
  ";" @append_hardline
  .
  [
    (block_comment)
    (line_comment)
  ]* @do_nothing
)

(param_list
  "," @append_spaced_softline
  .
  [(block_comment) (line_comment)]* @do_nothing
)

(use_names_list
  "," @append_spaced_softline
  .
  [(block_comment) (line_comment)]* @do_nothing
) @prepend_empty_softline

[
  ">"
  ")"
  ","
  ";"
] @prepend_antispace

"," @append_space

[
  "<"
  "("
] @append_antispace

(param_list
  .
  "(" @append_empty_softline @append_indent_start
  ")" @prepend_indent_end @prepend_empty_softline
  .
)

; Colon should have whitespace trimmed for URI separator
; pkg & use nodes
(package_decl
  ["@" ":" "/"] @prepend_antispace @append_antispace
)
(use_path
  ["@" ":" "/"] @prepend_antispace @append_antispace
)
(record_field
  ":" @prepend_antispace @append_space
)
(named_type
  ":" @prepend_antispace @append_space
)

[
  "@"
  "/"
  "."
] @prepend_antispace @append_antispace

; ==================
; Trailing Commas
; ==================
(use_names_list
  (#delimiter! ",")
  (use_names_item) @append_delimiter
  .
  ","? @do_nothing
  .
  (line_comment)*
  .
  (#multi_line_only!)
)

(param_list
  (#delimiter! ",")
  (named_type) @append_delimiter
  .
  ","? @do_nothing
  .
  (line_comment)*
  .
  ")"
  .
  (#multi_line_only!)
)

(variant_cases
  (#delimiter! ",")
  (variant_case) @append_delimiter
  .
  ","? @do_nothing
  .
  (line_comment)*
  .
  (#multi_line_only!)
)

(enum_cases
  (#delimiter! ",")
  (enum_case) @append_delimiter
  .
  ","? @do_nothing
  .
  (line_comment)*
  .
  (#multi_line_only!)
)

(flags_items
  (body
    (#delimiter! ",")
    (id) @append_delimiter
    .
    ","? @do_nothing
    .
    (line_comment)*
    .
    "}"
    .
    (#multi_line_only!)
  )
)
