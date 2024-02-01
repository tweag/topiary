[
  (atom)
  (string)
] @leaf

[
  (comment)
  (s_expression)
  (module)
] @allow_blank_line_before

[
  (atom)
  (comment)
  (module_identifier)
  (module_governance)
  (string)
] @prepend_space @append_space

(comment) @append_hardline

; Remove spaces following opening/preceding closing parens in all sexprs
(
  "(" @append_antispace
  ")" @prepend_antispace
)

;; Indentation

; We opt for idiomatic Lisp indentation of sexprs: The indentation
; block is opened after a context-specific atom in the sexpr (not
; necessarily the head element); then the closing paren falls on the
; last line of the sexpr, rather than having a line of its own.
; Something like this:
;
;   (foo bar
;      (quux xyzzy))
;
; Opposed to Algol-style formatting:
;
;   (
;     foo bar
;     (quux xyzzy)
;   )

; Modules
; Start the indentation block after the (module_governance) node
(module
  (module_governance) @append_spaced_softline @append_indent_start
  ")" @append_indent_end
)

; S-Expressions
; Start the indentation block after the head node
(s_expression
  (s_expression_head) @append_spaced_softline @append_indent_start
  ")" @append_indent_end
)
