(cf_tag_open) @append_hardline
(cf_tag_close) @append_hardline

(
  (cf_tag_open) @append_indent_start
  _
  (cf_tag_close) @prepend_indent_end
)

(cf_set_tag) @append_hardline
(cf_selfclose_tag) @append_hardline

(cf_var) @prepend_space
(assignment_expression) @prepend_space @append_space

(assignment_expression "=" @prepend_space @append_space)

(
  (cf_selfclose_tag_end) @prepend_delimiter
  (#delimiter! "/")
  (#not-match? @prepend_delimiter "/")
)
