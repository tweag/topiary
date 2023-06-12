;; Prototype Typst grammar

(
  [
    (string_literal)
  ]
) @leaf

(set_expression
 "set" @append_space
)

(show_expression
 "show"
 .
 ":" @append_space)

(let_declaration
  "let" @append_space
  .
  (identifier) @append_space
  .
  "=" @append_spaced_softline @append_indent_start

  ;; there doesn't seem to be a toplevel node for let declaration
  ;; bodies, this is probably overfitting

  (function_call) @append_indent_end
)

(arguments
   [
     (argument)
     (asssigned_argument)
   ] @append_delimiter
  .
  ")"
  (#delimiter! ",")
  (#multi_line_only!)
)

(arguments
  "(" @append_empty_softline @append_indent_start
  .
  [
   (argument)
   (asssigned_argument)
   "," @append_spaced_softline
  ]*
  .
  ")" @prepend_empty_softline @prepend_indent_end
)

(asssigned_argument
  (identifier)
  .
  ":" @append_space)

(source_file
  ;; Very hacky. It works though.
  (paragraph_break) @append_delimiter
  (#delimiter! "\n\n")
)
