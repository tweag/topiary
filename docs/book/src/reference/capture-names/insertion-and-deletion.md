# Insertion and deletion

## `@append_delimiter` / `@prepend_delimiter`

The matched nodes will have a delimiter appended (or, respectively,
prepended) to them. The delimiter must be specified using the predicate
`#delimiter!`.

Note that `@append_delimiter` is a generalisation of `@append_space`
(and, respectively, for prepending) where the delimiter is set to `" "`
(i.e., a space); see [horizontal spacing](horizontal-spacing.md#append_space--prepend_space).

> **Note**\
> A delimiter can be any string; it is not limited to a single
> character.

### Example

```scheme
; Put a semicolon delimiter after field declarations, unless they already have
; one, in which case we do nothing.
(
  (field_declaration) @append_delimiter
  .
  ";"* @do_nothing

  (#delimiter! ";")
)
```

If there is already a semicolon, the `@do_nothing` instruction (see
[general advice](general.md#do_nothing)) will be activated and prevent
the other instructions in the query (the `@append_delimiter`, here) from
applying. Otherwise, the `";"*` captures nothing and, in this case, the
associated instruction (`@do_nothing`) does not activate.

## `@delete`

Remove the matched node from the output.

### Example

```scheme
; Move semicolon after comments.
(
  ";" @delete
  .
  (comment)+ @append_delimiter

  (#delimiter! ";")
)
```

> **Note**\
> The above example uses a combination of `@delete` and
> `@append_delimiter` (see [above](#append_delimiter--prepend_delimiter))
> to effectively implement a rewrite rule.
