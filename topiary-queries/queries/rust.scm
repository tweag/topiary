; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
[
  (string_literal)
] @leaf

; Allow blank line before
[
  (attribute_item)
  (call_expression)
  (enum_item)
  (enum_variant)
  (field_declaration)
  (function_item)
  (impl_item)
  (let_declaration)
  (mod_item)
  (struct_item)
  (type_item)
  (use_declaration)
] @allow_blank_line_before

; Surround spaces
[
  "as"
  "const"
  "else"
  "extern"
  "fn"
  "for"
  "if"
  "let"
  (mutable_specifier)
  (scoped_use_list)
  "struct"
  "type"
  "unsafe"
  (visibility_modifier)
  "while"
  "="
  "=="
  "-"
  "+"
  "->"
] @prepend_space @append_space

; Append spaces
[
  ":"
] @append_space

; Append line breaks.
(
  [
    (attribute_item)
    (enum_item)
    (extern_crate_declaration)
    (expression_statement)
    (function_item)
    (impl_item)
    (let_declaration)
    (mod_item)
    (struct_item)
    (type_item)
    (use_declaration)
  ] @append_spaced_softline
)

; Append softlines
(
  [
    ","
    ";"
  ] @append_spaced_softline
)

; Prepend softlines before dots
(_
  "." @prepend_empty_softline
)

; enum
(enum_item
  (type_identifier) @prepend_space
)

; This patterns is duplicated for all nodes that can contain curly braces.
; Hoping to be able to generalise them like this:
; (_
;   .
;   "{" @prepend_space
;   (#for! block declaration_list enum_variant_list field_declaration_list)
; )
; Perhaps even the built in #match! can do this

(enum_variant_list
  .
  "{" @prepend_space
)

(enum_variant_list
  .
  "{" @append_spaced_softline @append_indent_start
  _
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

; field
(field_declaration_list
  .
  "{" @prepend_space
)

(field_declaration_list
  .
  "{" @append_spaced_softline @append_indent_start
  _
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

; fn
(function_item
  (identifier) @prepend_space
)

(block
  .
  "{" @prepend_space
)

(block
  .
  "{" @append_spaced_softline @append_indent_start
  "}" @prepend_spaced_softline @prepend_indent_end
  .
)

; impl
(impl_item
  (type_identifier) @prepend_space
)

(declaration_list
  .
  "{" @prepend_space
)

(declaration_list
  .
  "{" @append_empty_softline @append_indent_start
  "}" @prepend_empty_softline @prepend_indent_end
  .
)

; PhantomData<&'a ()>
(_
  (lifetime) @append_space
  [(array_type) (generic_type) (primitive_type) (unit_type)]
)

; Never put a space before a comma
(
  "," @prepend_antispace
)
