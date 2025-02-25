# Case

## `@lower_case` / `@upper_case`

Set the capitalisation of all text in the matched node and its children
(besides [designated leaf nodes](basics.md#leaves), which remain
unchanged).

<div class="warning">
Use this with care in languages that are case sensitive.
</div>

### Example

```scheme
; Example for SQL, since that's where this makes sense
; (Grammar: https://github.com/DerekStride/tree-sitter-sql)

; Make keywords "select" and "from" lowercase
[
  (keyword_select)
  (keyword_from)
] @lower_case

; Make keyword "WHERE" uppercase
(keyword_where) @upper_case
```
