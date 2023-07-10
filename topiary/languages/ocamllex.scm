; NOTE[regexp] regexp is a unnamed node without a field name, so we typically
; account for places it can be instead of formatting it directly.

[
  (ocaml)
  (character)
  (string)
] @leaf

(comment) @multi_line_indent_all
(comment) @prepend_input_softline @append_input_softline

(
  (#scope_id! "action")
  (action) @begin_scope @end_scope
)

; If the action spanned multiple lines, add newlines
(action
  (#scope_id! "action")
  "{" @append_spaced_scoped_softline @append_indent_start
  "}" @prepend_spaced_scoped_softline @prepend_indent_end
)

; Regular expression related rules
(named_regexp
  "let" @append_space
  "=" @prepend_space @append_space
  ; If the regexp spand multiple lines, we may want it to be indented
  ; regexp: (_) @prepend_indent_start @append_indent_end
) @allow_blank_line_before @append_hardline

; Actual regex rules
(character_set
  (#scope_id! "character_set")
  "[" @append_empty_scoped_softline @append_indent_start
  (_) @append_spaced_scoped_softline
  ; Do not append spaces to the last element, then only place a newline
  (_) @append_empty_scoped_softline
  .
  "]" @prepend_indent_end @prepend_antispace
) @begin_scope @end_scope

(parenthesized_regexp
  (#scope_id! "parenthesized_regexp")
  "(" @append_empty_scoped_softline @append_indent_start
  ")" @prepend_empty_scoped_softline @prepend_indent_end
) @begin_scope @end_scope

(parenthesized_regexp
  (#scope_id! "regexp_alternative")
  (regexp_alternative) @begin_scope @end_scope
)

(regexp_alternative
  (#scope_id! "regexp_alternative")
  "|" @prepend_spaced_scoped_softline @append_space
)

(lexer_definition
  [ "rule" "and" ] @allow_blank_line_before @append_space
)

(lexer_entry
  (#scope_id! "lexer_entry")
; TODO: Arguments
  "=" @prepend_space @append_space
  [ "parse" "shortest" ] @append_spaced_scoped_softline @append_indent_start
) @begin_scope @end_scope

(lexer_entry
  (#scope_id! "lexer_entry")
  ; @append_space is for NOTE[regexp]
  ; TODO: Is this allowed to be a space? Use hardline if not
  "|" @prepend_spaced_scoped_softline @append_space @allow_blank_line_before
  ; @prepend_space is for NOTE[regexp]
)

(lexer_entry
  ; End indentation after the last child node
  (_) @append_indent_end
  .
)

(lexer_case
  (action) @prepend_space @prepend_indent_start @append_indent_end
)
