; Sometimes we want to indicate that certain parts of our source text should
; not be formated, but taken as is. We use the leaf capture name to inform the
; tool of this.
(string_literal) @leaf

; Append line breaks
[
  (attribute_item)
  (enum_item)
  (function_item)
  (line_comment)
  (struct_item)
  (use_declaration)
] @append_hardline

; Allow blank line before
[
  (attribute_item)
  (enum_item)
  (enum_variant)
  (field_declaration)
  (function_item)
  (line_comment)
  (struct_item)
  (use_declaration)
] @allow_blank_line_before

; Append spaces
[
  (visibility_modifier)
] @append_space

; Prepend spaces
[
  (scoped_use_list)
] @prepend_space

; dyn
(dynamic_type
  (scoped_type_identifier) @prepend_space
)

; enum
(enum_item
  (type_identifier) @prepend_space
)

(enum_item
  (type_identifier) @append_space
)

(enum_variant_list
  (",") @append_softline
  (enum_variant)
)

(enum_variant_list
  ("{") @append_softline
)

(enum_variant_list
  ("}") @prepend_softline
)

; field

(field_declaration_list
  (",") @append_softline
  (field_declaration)
)

(field_declaration_list
  ("{") @prepend_space
)

(field_declaration_list
  ("{") @append_softline
)

(field_declaration_list
  ("}") @prepend_softline
)

(field_declaration
  (":") @append_space
)

; fn
(function_item
  (identifier) @prepend_space
)

(function_item
  (identifier) @append_space
)

; for
(for_expression
  (identifier) @prepend_space
)

(for_expression
  (identifier) @append_space
)

(for_expression
  (call_expression) @prepend_space
)

(for_expression
  (field_expression) @prepend_space
)

; if
(if_expression
  (binary_expression) @prepend_space
)

; if let
(if_let_expression
  "let" @prepend_space
)

(if_let_expression
  (identifier) @prepend_space
)

; let
(let_declaration
  (identifier) @prepend_space
)

(let_declaration
  (mutable_specifier) @prepend_space
)

; match
(match_expression
  (call_expression) @prepend_space
)

(match_expression
  (field_expression) @prepend_space
)

(match_expression
  (identifier) @prepend_space
)

; mut
(
  (mutable_specifier) @append_space
  .
  (dynamic_type)
)

; return
(return_expression
  (identifier) @prepend_space
)

; struct
(struct_item
  (type_identifier) @prepend_space
)

(struct_item
  (type_identifier) @append_space
)

; use
(use_declaration
  (scoped_identifier) @prepend_space
)
