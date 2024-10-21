;; Nix Formatter based on RFC 166

; Leaf nodes
[
  (string_expression)
  (integer_expression)
  (float_expression)
  (uri_expression)
  (path_expression)
  (hpath_expression)
  (spath_expression)
] @leaf

; Comments
(comment) @comment

; Convert single-line /* */ comments to #
(comment) @convert_to_hash_comment

; Preserve multiline comments
(comment) @preserve_multiline_comment

; Append space after specific tokens
":" @append_space
"=" @prepend_space @append_space
"," @append_spaced_softline

; Handle indentation and line breaks
; Define softline breaks and indentation starts/ends
"{" @append_spaced_softline @append_indent_start
"}" @prepend_spaced_softline @prepend_indent_end
"[" @append_spaced_softline @append_indent_start
"]" @prepend_spaced_softline @prepend_indent_end

; Functions

(function_expression
  formals: (formals
    "{" @append_spaced_softline @append_indent_start
    (formal) @formal_param
    "}" @prepend_spaced_softline @prepend_indent_end
  )
  body: (_) @function_body
) @function_declaration

; Ensure each parameter is on a new line with proper indentation
(formal) @prepend_spaced_softline

; Attribute Sets

(attrset_expression
  "{" @append_spaced_softline @append_indent_start
  (binding_set
    (binding) @attr_binding
  )
  "}" @prepend_spaced_softline @prepend_indent_end
) @attribute_set

; Ensure each key-value pair is on its own line
(binding
  attrpath: (_) @attr_key
  "=" @append_space
  (_) @attr_value
  ";" @append_spaced_softline
)

; Lists

(list_expression
  "[" @append_spaced_softline @append_indent_start
  (_) @list_item
  "]" @prepend_spaced_softline @prepend_indent_end
) @list

; If-Then-Else Expressions

(if_expression
  "if" @keyword_if @append_space @prepend_spaced_softline
  condition: (_) @if_condition
  "then" @keyword_then @append_space @prepend_spaced_softline @append_indent_start
  ;body: (_) @if_body @append_indent_end
  "else" @keyword_else @append_space @prepend_spaced_softline @append_indent_start
  alternative: (_) @else_body @append_indent_end
)

; Let-In Expressions

(let_expression
  "let" @keyword_let @append_spaced_softline @append_indent_start
  (binding_set
    (binding
      attrpath: (_) @let_binding_name
      "=" @append_space
      (_) @let_binding_value
      ";" @append_spaced_softline
    )+
  )
  "in" @keyword_in @prepend_spaced_softline @prepend_indent_end @append_spaced_softline
  body: (_) @let_body
)

; With Expressions

(with_expression
  "with" @keyword_with @append_space
  (_) @with_expression
  ";" @append_spaced_softline
  body: (_) @with_body
)

; Assert Expressions

(assert_expression
  "assert" @keyword_assert @append_space
  (_) @assert_condition
  ";" @append_spaced_softline
  body: (_) @assert_body
)

; Operator Formatting

(binary_expression
  left: (_) @op_left
  right: (_) @op_right
)

; Ensure operators start on new lines if they don't fit on a single line
(binary_expression
  (_) @prepend_spaced_softline
)

; Inherit Statements

(inherit
  "inherit" @keyword_inherit @append_space
  (inherited_attrs
    (identifier) @inherit_item
  )+
  ";" @inherit_semicolon
)

; Handle inherit with source
(inherit_from
  "inherit" @keyword_inherit @append_space
  "(" @inherit_parentheses_start
  (_) @inherit_source
  ")" @inherit_parentheses_end
  (inherited_attrs
    (identifier) @inherit_item
  )+
  ";" @inherit_semicolon
)

; Empty Objects and Arrays

(attrset_expression
  "{" @object_open
  "}" @object_close
) @empty_attribute_set

(list_expression
  "[" @array_open
  "]" @array_close
) @empty_array

; String Interpolation

(interpolation
  "${" @interp_start
  (_) @interp_content
  "}" @interp_end
)

; Additional Rules

; Handle let bindings indentation
(let_expression
  (binding_set
    (binding) @let_binding_indent
  )
)

; Handle assertions
(assert_expression
  "assert" @keyword_assert @append_space
  (_) @assert_condition
  ";" @assert_semicolon
)

; Handle comments within expressions
(comment) @comment_within_expr

; Leaf Nodes Preservation
(_) @leaf

; Handle semicolon placement in bindings
(binding
  ";" @semicolon_end
)

; Handle indentation for nested attribute sets
(attrset_expression
  (binding_set
    (binding
      (attrset_expression) @nested_attribute_set
    )
  )
)

; Handle newline preservation rules
(ERROR) @collapse_empty_lines
