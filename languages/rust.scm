; Configuration
(#language! rust)
(#indent-level! 4)

; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
(string_literal) @leaf

; Allow blank line before
[
  (attribute_item)
  (block_comment)
  (call_expression)
  (enum_item)
  (enum_variant)
  (field_declaration)
  (function_item)
  (impl_item)
  (let_declaration)
  (line_comment)
  (mod_item)
  (struct_item)
  (type_item)
  (use_declaration)
] @allow_blank_line_before

; Append spaces
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
  "struct"
  "type"
  "unsafe"
  (visibility_modifier)
  "="
  "=="
  "-"
  "+"
  "->"
  ":"
] @append_space

; Prepend spaces
[
  "as"
  "else"
  "extern"
  "fn"
  "for"
  "if"
  "let"
  (scoped_use_list)
  "unsafe"
  "="
  "=="
  "-"
  "+"
  "->"
] @prepend_space

; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. A line comment
; always ends with a line break.
(block_comment) @prepend_input_softline
(line_comment) @prepend_input_softline

; Input softline after block comments unless followed by comma or semicolon, as
; they are always put directly after.
(
  (block_comment) @append_input_softline
  .
  [ "," ";" ]* @do_nothing
)

; Append line breaks. If there is a comment following, we don't add anything,
; because the input softlines and spaces above will already have sorted out the
; formatting.
((attribute_item)           @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)
((enum_item)                @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)
((extern_crate_declaration) @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)
((function_item)            @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)
((impl_item)                @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)
((let_declaration)          @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)
((line_comment)             @append_hardline)
((mod_item)                 @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)
((struct_item)              @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)
((type_item)                @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)
((use_declaration)          @append_hardline . [ (block_comment) (line_comment) ]* @do_nothing)

; Append softlines
[
  ";"
] @append_spaced_softline

; Append softlines after commas, unless followed by comments.
(
  "," @append_spaced_softline
  . 
  [ (block_comment) (line_comment) ]* @do_nothing
)

; Prepend softlines before dots
(_
  "." @prepend_empty_softline
)

; dyn
(dynamic_type
  (scoped_type_identifier) @prepend_space
)

; enum
(enum_item
  (type_identifier) @prepend_space
)

; The following five patterns are duplicated for all nodes that can contain curly braces.
; Hoping to be able to generalise them like this:
; (enum_variant_list
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
  "{" @append_spaced_softline
  _
  "}"
)

(enum_variant_list
  .
  "{" @append_indent_start
  _ 
  "}"
)

(enum_variant_list
  "{"
  _
  "}" @prepend_indent_end
  .
)

(enum_variant_list
  "{"
  _
  "}" @prepend_spaced_softline
  .
)

; extern
(extern_crate_declaration
  (crate) @prepend_space
)

(extern_crate_declaration
  (identifier) @prepend_space
)

; field
(field_declaration_list
  .
  "{" @prepend_space
)

(field_declaration_list
  .
  "{" @append_spaced_softline
  _
  "}"
)

(field_declaration_list
  .
  "{" @append_indent_start
  _ 
  "}"
)

(field_declaration_list
  "{"
  _
  "}" @prepend_indent_end
  .
)

(field_declaration_list
  "{"
  _
  "}" @prepend_spaced_softline
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
  "{" @append_spaced_softline
  _
  "}"
)

(block
  .
  "{" @append_indent_start
  _ 
  "}"
)

(block
  "{"
  _
  "}" @prepend_indent_end
  .
)

(block
  "{"
  _
  "}" @prepend_spaced_softline
  .
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
  "{" @append_spaced_softline
  _
  "}"
)

(declaration_list
  .
  "{" @append_indent_start
  _ 
  "}"
)

(declaration_list
  "{"
  _
  "}" @prepend_indent_end
  .
)

(declaration_list
  "{"
  _
  "}" @prepend_spaced_softline
  .
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

; mod
(mod_item
  (identifier) @prepend_space
)

; return
(return_expression
  (identifier) @prepend_space
)

; type
(bounded_type
  "+" @prepend_space
)

(bounded_type
  "+" @append_space
)

; PhantomData<&'a ()>
(_
  (lifetime) @append_space
  [(array_type) (generic_type) (primitive_type) (unit_type)]
)

; use
(use_declaration
  (scoped_identifier) @prepend_space
)
