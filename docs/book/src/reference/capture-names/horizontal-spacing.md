# Horizontal spacing

This chapter discusses horizontal spacing explicitly. However,
horizontal spacing can also be introduced with softlines (see [vertical
spacing](vertical-spacing.md#append_spaced_softline--prepend_spaced_softline))
and indentation (see [indentation](indentation.md)).

## `@append_space` / `@prepend_space`

The matched nodes will have a space appended (or, respectively,
prepended) to them. Note that this is the same as `@append_delimiter` /
`@prepend_delimiter`, with a space as the delimiter (see [insertion and
deletion](insertion-and-deletion.md#append_delimiter--prepend_delimiter)).

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
`@append_antispace` / `@prepend_antispace`; this will destroy all
horizontal whitespace (besides any added through indentation) from that
node, including those added by other formatting rules.

### Example

```scheme
[
  ","
  ";"
  ":"
  "."
] @prepend_antispace
```
