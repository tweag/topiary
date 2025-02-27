# Horizontal spacing

<!----------------------------------------------------------------------
TODO: Mention spaced softlines, but these should be discussed in
vertical spacing chapter.
----------------------------------------------------------------------->

## `@append_space` / `@prepend_space`

The matched nodes will have a space appended or prepended to them. Note
that this is the same as `@append_delimiter` / `@prepend_delimiter`,
with a space as the delimiter (see [insertion and deletion](insertion-and-deletion.md#append_delimiter--prepend_delimiter)).

### Example

```scheme
[
  (infix_operator)
  "if"
  ":"
] @append_space
```

## `@append_antispace` / `@prepend_antispace`

It is often the case that tokens need to be juxtaposed with spaces,
except in a few isolated contexts. Rather than writing complicated rules
that enumerate every exception, an "antispace" can be inserted with
`@append_antispace` / `@prepend_antispace`; this will destroy any spaces
(not newlines) from that node, including those added by other formatting
rules.

### Example

```scheme
[
  ","
  ";"
  ":"
  "."
] @prepend_antispace
```
