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
  (exception_definition)
  (module_definition)
  (open_module)
  (type_definition)
  (value_definition)
] @allow_blank_line_before

; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. But don't put a
; softline directly in front of commas or semicolons.

(comment) @prepend_input_softline

(
  (comment) @append_input_softline
  .
  [ "," ";" ]* @do_nothing
)

; Append line breaks. If there is a comment following, we don't add anything,
; because the input softlines and spaces above will already have sorted out the
; formatting.
(
  [
    (exception_definition)
    (external)
    (module_definition)
    (module_type_definition)
    (open_module)
    (type_definition)
  ] @append_hardline
  .
  (comment)* @do_nothing
)

; Consecutive definitions must be separated by line breaks
(
  (value_definition) @append_hardline
  .
  [
    (exception_definition)
    (external)
    (module_definition)
    (module_type_definition)
    (open_module)
    (type_definition)
    (value_definition)
  ]
)

; Surround spaces
; A space is put after, and before (except just after an open parenthesis).
[
  "as"
  "assert"
  "begin"
  "do"
  "else"
  "exception"
  "external"
  (infix_operator)
  "let"
  "match"
  "module"
  "mutable"
  "of"
  "open"
  (parameter)
  "rec"
  "sig"
  "then"
  "try"
  "type"
  "val"
  "when"
  "while"
  "with"
  "*"
  "="
  "|"
  "||"
  "->"
  "<-"
  "{"
  "}"
] @append_space

(
  "("* @do_nothing
  .
  [
    "as"
    "assert"
    "begin"
    "do"
    "else"
    "exception"
    "external"
    (infix_operator)
    "let"
    "match"
    "module"
    "mutable"
    "of"
    "open"
    (parameter)
    "rec"
    "sig"
    "then"
    "try"
    "type"
    "val"
    "when"
    "while"
    "with"
    "*"
    "="
    "|"
    "||"
    "->"
    "<-"
    "{"
    "}"
  ] @prepend_space
)

; Prepend spaces
[
  "done"
] @prepend_space

; Append spaces
[
  "if"
  ":"
  ";"
] @append_space

; Put a space after commas, except the last one.
(
  "," @append_space
  .
  (_)
)

; Certain elements must be separated by spaces.
(
  [
    (character)
    (constructed_type)
    (constructor_path)
    (field_get_expression)
    (labeled_argument)
    (number)
    (parenthesized_expression)
    (parenthesized_pattern)
    (string)
    (type_constructor_path)
    (typed_expression)
    (value_path)
    (value_pattern)
    ")"
  ] @append_space
  .
  [
    ; Don't add any space just before these.
    ","
    "."
    ".."
    ")"
  ]* @do_nothing
  .
  [
    (character)
    (constructed_type)
    (constructor_path)
    (field_get_expression)
    (labeled_argument)
    (number)
    (parenthesized_expression)
    (parenthesized_pattern)
    (prefix_expression)
    (string)
    (type_constructor_path)
    (typed_expression)
    (value_path)
    (value_pattern)
    "("
  ]
)

; Some kinds of colons should have a space in front.
(
  [
    (constructor_name)
    (fun_expression)
    (parenthesized_expression)
    (value_name)
    (value_pattern)
  ] @append_space
  .
  ":"
)

; Softlines. These become either a space or a newline, depending on whether we
; format their node as single-line or multi-line. If there is a comment
; following, we don't add anything, because they will have their own line break
; processing applied to them.
(
  [
    "begin"
    "else"
    "of"
    "struct"
    "then"
    "->"
    "{"
    ":"
    ";"
  ] @append_spaced_softline
  .
  [
    (comment)
  ]* @do_nothing
)

; Always put softlines before these:
[
  "end"
  (else_clause)
  (infix_operator)
  (match_expression)
  "*"
  "|"
  "}"
] @prepend_spaced_softline

; Softline before the first match case
;
; let type_constant = function
;   Const_int _ -> instance Predef.type_int
;   | Const_char _ -> instance Predef.type_char
;
(
  "|"* @do_nothing
  (match_case) @prepend_spaced_softline
)

; Multi-line definitions must have a linebreak before "in":
;
; let a =
;   expression
;   in
;   expression
;
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

(variant_declaration) @prepend_indent_start @append_indent_end

; Start an indented block after these
[
  "begin"
  "else"
  "sig"
  "struct"
  "then"
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

; Make an indented block after ":" in typed expressions
;
; (fun _env _md -> assert false) :
;   Env.t -> Parsetree.module_expr -> Typedtree.module_expr * Shape.t
;
(typed_expression
  ":" @append_indent_start
  (_) @append_indent_end
  .
  ; just doing _ above doesn't work, because it matches the final named node as
  ; well as the final non-named node, causing double indentation.
)

; Make an indented block after ":" in let bindings
;
; let type_open :
;   (?used_slot:bool ref -> override_flag -> Env.t -> Location.t ->
;
(let_binding
  ":" @append_indent_start
  (_) @append_indent_end
  .
)

; Make an indented block after "=" in let bindings
;
(let_binding
  "=" @append_indent_start
  (_) @append_indent_end
  .
)

; Make an indented block after "of" or ":" in constructor declarations
;
; | Expr_type_clash of
;   Errortrace.unification_error * type_forcing_context option
; | Pattern_type_clash :
;   Errortrace.unification_error * Parsetree.pattern_desc option -> error
;
(constructor_declaration
  [
    "of"
    ":"
  ] @append_indent_start
  (_) @append_indent_end
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

; Make an indented block after match arrows.
;
; match lid with
; | Ldot(_, "false") ->
;   Boolean
;
(match_case
  "->" @append_indent_start
  (_) @append_indent_end
  .
)

; Make an indented block after constructor declaration arrows.
;
; Errortrace.unification_error * Parsetree.pattern_desc option ->
;   error
;
(constructor_declaration
  "->" @append_indent_start
  (_) @append_indent_end
  .
)

; Make an indented block where a function type arrow starts. Only for the root
; level, not for each arrow.
;
; (?used_slot:bool ref ->
;   Longident.t loc ->
;   Path.t * Env.t)
;
(constructed_type
  (function_type
    "->" @append_indent_start
    (_) @append_indent_end
    .
  )
)

; Try block formatting
(
  "try" @append_spaced_softline @append_indent_start
  "with" @prepend_indent_end @prepend_spaced_softline @append_indent_start
  (_) @append_indent_end
  .
)

; Softlines and indenting between parenthesized expressions
;
; mkexp (Texp_construct(mknoloc lid, csome, [texp]))
;   (type_option texp.exp_type) texp.exp_loc texp.exp_env
;
(
  (parenthesized_expression) @append_spaced_softline @append_indent_start
  (parenthesized_expression) @append_indent_end
  .
)
