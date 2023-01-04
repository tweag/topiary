; Configuration
(#language! ocaml)

; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
(
  [
    (character)
    (quoted_string)
    (signed_number)
    (string)
  ]
) @leaf

; Allow blank line before
[
  (class_definition)
  (class_type_definition)
  (comment)
  (exception_definition)
  (external)
  (floating_attribute)
  (include_module)
  (include_module_type)
  (inheritance_definition)
  (inheritance_specification)
  (instance_variable_definition)
  (method_definition)
  (module_definition)
  (module_type_definition)
  (open_module)
  (type_definition)
  (value_definition)
  (value_specification)
] @allow_blank_line_before

; In a definition including several mutually recursive functions,
; one can skip a line before each of them.
(value_definition
  "and" @allow_blank_line_before
)
(value_definition
  (and_operator) @allow_blank_line_before
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

; Append line breaks. If there is a comment following, we don't add anything,
; because the input softlines and spaces above will already have sorted out the
; formatting.
(
  [
    (exception_definition)
    (external)
    (include_module)
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
(
  [
    "and"
    "as"
    "assert"
    (attribute)
    "class"
    "downto"
    "else"
    "exception"
    "external"
    (floating_attribute)
    "for"
    "if"
    "in"
    "include"
    (infix_operator)
    "inherit"
    (item_attribute)
    "let"
    "match"
    "method"
    "module"
    (module_parameter)
    "mutable"
    "new"
    "nonrec"
    "object"
    "of"
    "open"
    (parameter)
    "private"
    "rec"
    "sig"
    "then"
    "to"
    "try"
    "type"
    "val"
    "virtual"
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
    ":"
    ";"
    "+="
    ":="
    ":>"
  ] @append_space
  .
  "%"? @do_nothing
)

; Those keywords are not expected to come right after an open parenthesis.
[
    "as"
    "do"
    "done"
    "downto"
    "else"
    "in"
    "nonrec"
    "of"
    "rec"
    "then"
    "to"
    "virtual"
    "when"
    "with"
    "|"
    "->"
    "<-"
    "+="
    ":="
    ":>"
] @prepend_space

; let-like and and-like operators are only followed by a closing parenthesis
; during their definition, in which case no space must be appended.
; space must be appended otherwise
(
  (and_operator) @append_space
  .
  ")"* @do_nothing
)
(
  (let_operator) @append_space
  .
  ")"* @do_nothing
)

; For those queries, we should not have multiple queries,
; however, due to a known bug in tree-sitter queries
; https://github.com/tree-sitter/tree-sitter/issues/1811
; using an alternative after the starred parenthesis does not work as intented.
;
(
  "("* @do_nothing
  .
  "assert" @prepend_space
)
(
  "("* @do_nothing
  .
  (attribute) @prepend_space
)
(
  "("* @do_nothing
  .
  "begin" @prepend_space
)
(
  "("* @do_nothing
  .
  "class" @prepend_space
)
(
  "("* @do_nothing
  .
  "exception" @prepend_space
)
(
  "("* @do_nothing
  .
  "external" @prepend_space
)
(
  "("* @do_nothing
  .
  (floating_attribute) @prepend_space
)
(
  "("* @do_nothing
  .
  "for" @prepend_space
)
(
  "("* @do_nothing
  .
  "include" @prepend_space
)
(
  "("* @do_nothing
  .
  (infix_operator) @prepend_space
)
(
  "("* @do_nothing
  .
  "inherit" @prepend_space
)
(
  "("* @do_nothing
  .
  (item_attribute) @prepend_space
)
(
  "("* @do_nothing
  .
  "let" @prepend_space
)
(
  "("* @do_nothing
  .
  "match" @prepend_space
)
(
  "("* @do_nothing
  .
  "method" @prepend_space
)
(
  "("* @do_nothing
  .
  "module" @prepend_space
)
(
  "("* @do_nothing
  .
  (module_parameter) @prepend_space
)
(
  "("* @do_nothing
  .
  "mutable" @prepend_space
)
(
  "("* @do_nothing
  .
  "new" @prepend_space
)
(
  "("* @do_nothing
  .
  "object" @prepend_space
)
(
  "("* @do_nothing
  .
  "open" @prepend_space
)
(
  "("* @do_nothing
  .
  (parameter) @prepend_space
)
(
  "("* @do_nothing
  .
  "private" @prepend_space
)
(
  "("* @do_nothing
  .
  "sig" @prepend_space
)
(
  "("* @do_nothing
  .
  "try" @prepend_space
)
(
  "("* @do_nothing
  .
  "type" @prepend_space
)
(
  [
    "("
    "["
  ]* @do_nothing
  .
  (type_variable) @prepend_space
)
(
  "("* @do_nothing
  .
  "val" @prepend_space
)
(
  "("* @do_nothing
  .
  "while" @prepend_space
)
(
  "("* @do_nothing
  .
  "*" @prepend_space
)
(
  "("* @do_nothing
  .
  "=" @prepend_space
)
(
  "("* @do_nothing
  .
  "||" @prepend_space
)
(
  "("* @do_nothing
  .
  "}" @prepend_space
)

; Put a space after commas, except the last one.
(
  "," @append_space
  .
  (_)
)

; Put a space after the dot in polymorphic function types
;
; let my_const :
;   type a b. a: a -> b: b -> a =
;   fun ~a ~b -> a
(polymorphic_type
  (abstract_type)
  .
  "." @append_space
)

; Certain elements must be separated by spaces.
(
  [
    (boolean)
    (character)
    (class_path)
    (class_type_path)
    (constructed_type)
    (constructor_path)
    (extended_module_path)
    (field_get_expression)
    (labeled_argument)
    (module_path)
    (number)
    (parenthesized_expression)
    (parenthesized_pattern)
    (quoted_string)
    (string)
    (tag)
    (type_constructor_path)
    (typed_expression)
    (type_variable)
    (value_path)
    (value_pattern)
    ")"
    "]"
  ] @append_space
  .
  [
    ; Don't add any space just before these.
    ","
    "."
    ".."
    ")"
    "]"
    "::"
  ]* @do_nothing
  .
  [
    (boolean)
    (character)
    (class_name)
    (class_path)
    (class_type_name)
    (class_type_path)
    (constructed_type)
    (constructor_path)
    (constructor_pattern)
    (extended_module_path)
    (field_get_expression)
    (local_open_pattern)
    (labeled_argument)
    (module_path)
    (number)
    (parenthesized_expression)
    (parenthesized_pattern)
    (prefix_expression)
    (quoted_string)
    (range_pattern)
    (string)
    (tag)
    (type_constructor_path)
    (typed_expression)
    (value_name)
    (value_path)
    (value_pattern)
    "("
    "{"
  ]
)

; When one uses a language extension, we always put a space after the extension name,
; if this name ends by the '?' or ':' token, the space comes after it.
;
(
  (attribute_id) @append_space
  .
  (attribute_payload
    [
      "?"
      ":"
    ]
  )* @do_nothing
)
(attribute_payload
  [
    "?"
    ":"
  ] @append_space
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

; Space is the application operator.
; There should be a space between a function and its first argument,
; as well as between all of its arguments.
(application_expression
  (_) @append_space
  .
  (_)
)

; There are various situation where type application can occur.
; In each of them, we want to separate the arguments with spaces.
(abstract_type
  [
    "type"
    (type_constructor)
  ] @append_space
  .
  (type_constructor)
)
(constructed_type
  (_) @append_space
  .
  (_)
)
(type_binding
  (_) @append_space
  .
  (_)
)
(constrain_type
  (_) @append_space
  .
  (_)
)

; Keep spacing between pointy brackets when used in object types.
;
; This is syntactically correct:
;   let obj_id (obj : < .. >) = obj
; This is not:
;   let obj_id (obj : <..>) = obj
(object_type
  "<" @append_space
  (_)*
  ">" @prepend_space
)

; Keep spacing around the list constructor if it is preceded by
; a labeled argument, avoiding syntax errors.
;
; This is syntactically correct:
;   foo ~arg :: []
; This is not:
;   foo ~arg::[]
;
; While both are correct if the argument is not labeled
(
  (application_expression
    (labeled_argument)
    .
  )
  .
  "::" @prepend_space @append_space
)

; Softlines. These become either a space or a newline, depending on whether we
; format their node as single-line or multi-line. If there is a comment
; following, we don't add anything, because they will have their own line break
; processing applied to them.
;
; If those keywords are followed by a ppx extension,
; the soft linebreak comes after it.
(
  [
    "begin"
    "do"
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
    "%"
  ]* @do_nothing
)

(
  [
    "begin"
    "do"
    "else"
    "in"
    "of"
    "struct"
    "then"
    "with"
    "->"
    "{"
    ":"
    ";"
  ]
  .
  "%"
  .
  (attribute_id) @append_spaced_softline
  .
  (comment)* @do_nothing
)

; Always put softlines before these:
[
  "and"
  "done"
  "end"
  (else_clause)
  (infix_operator)
  (item_attribute)
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
  .
  (match_case) @prepend_spaced_softline
)

; Multi-line definitions must have a linebreak after "=" and before "in":
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
; There are special cases however. We do not want to break lines after "=" when writing
;
; let f = function
;   | Constructor -> expression
;
; or
;
; let f = fun x ->
;   expression
;
(let_binding
  "=" @append_spaced_softline
  .
  [
    (function_expression)
    (fun_expression)
  ]* @do_nothing
)

(value_definition
  (and_operator) @prepend_spaced_softline
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
    (class_body_type)
    (if_expression)
    (let_expression)
    (object_expression)
    (product_expression)
    (record_expression)
    (sequence_expression)
    (set_expression)
    (typed_expression)
    (value_path)
  ]
)

; In module signature, each symbol declaration is separated by a softline.
;
; module type Name = sig
;   val function1 : type1
;   val function2 : type2
;  end
;
(signature
  "sig" @append_spaced_softline
  [
    (value_specification)
    (type_definition)
    (include_module_type)
  ] @append_spaced_softline
)

; In class definitions and class type definitions, each declaration is separated
; by a softline.

; class foo =
;   object
;     inherit bar
;     val baz = None
;     method qux = None
;   end
;
(object_expression
  "object"
  [
    (inheritance_definition)
    (instance_variable_definition)
    (method_definition)
  ] @append_spaced_softline @prepend_spaced_softline
)

(class_body_type
  "object"
  [
    (inheritance_specification)
    (instance_variable_definition)
    (method_definition)
  ] @append_spaced_softline @prepend_spaced_softline
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

(
  (variant_declaration) @prepend_indent_start @append_indent_end
)

; Start an indented block after these
[
  "begin"
  "do"
  "else"
  "object"
  "sig"
  "struct"
  "then"
] @append_indent_start

; "{" can be used to start quoted strings. Don't indent in that case
(
  "{" @append_indent_start
  .
  (quoted_string_content)* @do_nothing
)

; End the indented block before these
(
  [
    "done"
    "end"
  ] @prepend_indent_end
)

; "}" can be used to end quoted strings. Don't indent in that case
(
  (quoted_string_content)* @do_nothing
  .
  "}" @prepend_indent_end
)

; End the indented block after these
(
  [
    (else_clause)
    (then_clause)
  ] @append_indent_end
)

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

; Make an indented block after "=" in
; * let bindings
; * class[_type] bindings
; * method definitions
; * instance variable definitions
;
(let_binding
  "=" @append_indent_start
  (_) @append_indent_end
  .
)

(class_binding
  "=" @append_indent_start
  (_) @append_indent_end
)

(class_type_binding
  "=" @append_indent_start
  (_) @append_indent_end
)

(method_definition
  "=" @append_indent_start
  (_) @append_indent_end
)

(instance_variable_definition
  "=" @append_indent_start
  (_) @append_indent_end
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
; A soft linebreak after the "try" (potentially "try%ppx") and one after the "with".
(try_expression
  "try" @append_spaced_softline @append_indent_start
  .
  "%"* @do_nothing
)
(try_expression
  "try"
  .
  "%"
  .
  (attribute_id) @append_spaced_softline @append_indent_start
)
(try_expression
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
