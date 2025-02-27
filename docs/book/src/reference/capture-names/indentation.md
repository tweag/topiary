# Indentation

## `@append_indent_start` / `@prepend_indent_start`

The matched nodes will trigger indentation before or after them. This
will only apply to lines following, until an indentation end is
signalled. If indentation is started and ended on the same line, nothing
will happen. This is useful, because we get the correct behaviour
whether a node is formatted as single-line or multi-line. It is
important that all indentation starts and ends are balanced.

### Example

```scheme
; Start an indented block after these
[
  "begin"
  "else"
  "then"
  "{"
] @append_indent_start
```

## `@append_indent_end` / `@prepend_indent_end`

The matched nodes will trigger that indentation ends before or after
them.

### Example

```scheme
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
```

## `@multi_line_indent_all`

To be used on comments or other leaf nodes, to indicate that we should indent
all its lines, not just the first.

### Example

```scheme
(#language! ocaml)
(comment) @multi_line_indent_all
```

## `@single_line_no_indent`

The matched node will be printed alone, on a single line, with no indentation.

### Example

```scheme
(#language! ocaml)
; line number directives must be alone on their line, and can't be indented
(line_number_directive) @single_line_no_indent
```
