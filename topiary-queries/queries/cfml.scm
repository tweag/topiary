(cf_component
  ">" @append_hardline
)

(cf_set_tag) @append_hardline
  
(assignment_expression) @prepend_space @append_space

(assignment_expression "=" @prepend_space @append_space)

(
  (cf_selfclose_tag_end) @prepend_delimiter
  (#delimiter! "/")
  (#not-match? @prepend_delimiter "/")
)
