; Sometimes we want to indicate that certain parts of our source text should
; not be formatted, but taken as is. We use the leaf capture name to inform the
; tool of this.
[
  (block_comment)
  (line_comment)
] @leaf

; Allow blank line before
[
  (interface_item)
  (type_item)
  (enum_items)
  (variant_items)
] @allow_blank_line_before
