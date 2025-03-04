# Scopes

So far, we've expanded softlines into line breaks depending on whether
the CST node they are associated with is multi-line. Sometimes, CST
nodes define scopes that are either too big or too small for our needs.
For instance, consider this piece of OCaml code:

```ocaml
(1,2,
3)
```

Its CST is the following:

```
{Node parenthesized_expression (0, 0) - (1, 2)} - Named: true
  {Node ( (0, 0) - (0, 1)} - Named: false
  {Node product_expression (0, 1) - (1, 1)} - Named: true
    {Node product_expression (0, 1) - (0, 4)} - Named: true
      {Node number (0, 1) - (0, 2)} - Named: true
      {Node , (0, 2) - (0, 3)} - Named: false
      {Node number (0, 3) - (0, 4)} - Named: true
    {Node , (0, 4) - (0, 5)} - Named: false
    {Node number (1, 0) - (1, 1)} - Named: true
  {Node ) (1, 1) - (1, 2)} - Named: false
```

We would like to add a line break after the first comma, but because the
CST structure is nested, the node containing this comma
(`product_expression (0, 1) - (0, 4)`) is *not* multi-line  Only the
top-level node `product_expression (0, 1) - (1, 1)` is multi-line.

To solve this issue, we introduce user-defined scopes and softlines.

> **Note**\
> Similar to the non-scoped single- and multi-lined context, the scoped
> equivalents can also be used to drive arbitrary queries, not
> necessarily related to vertical spacing; see [below](#testing-context-with-predicates).

## `@append_begin_scope` / `@prepend_begin_scope` / `@append_end_scope` / `@prepend_end_scope`

These capture names are used to define custom scopes. In conjunction
with the `#scope_id!` predicate, they define scopes that can span
multiple CST nodes, or only part of one. For instance, this scope
matches anything between parenthesis in a `parenthesized_expression`:

```scheme
(parenthesized_expression
  "(" @append_begin_scope
  ")" @prepend_end_scope

  (#scope_id! "tuple")
)
```

### Scoped softlines

We have four predicates that insert softlines in custom scopes, in
conjunction with the `#scope_id!` predicate:

- [`@append_empty_scoped_softline` / `@prepend_empty_scoped_softline`](#append_empty_scoped_softline--prepend_empty_scoped_softline)
- [`@append_spaced_scoped_softline` / `@prepend_spaced_scoped_softline`](#append_spaced_scoped_softline--prepend_spaced_scoped_softline)

When one of these scoped softlines is used, their behaviour depends on
the innermost encompassing scope with the corresponding `scope_id`. If
that scope is multi-line, the softline expands into a line break. In any
other context, they behave as their non-scoped counterparts.

#### Example

This Tree-sitter query:

```scheme
(parenthesized_expression
  "(" @append_begin_scope @append_empty_softline @append_indent_start
  ")" @append_end_scope @prepend_empty_softline @prepend_indent_end

  (#scope_id! "tuple")
)

(product_expression
  "," @append_spaced_scoped_softline

  (#scope_id! "tuple")
)
```

formats this snippet of OCaml code:

```ocaml
(1,2,
3)
```

as:

```ocaml
(
  1,
  2,
  3
)
```

Whereas the single-lined `(1, 2, 3)` is kept as is.

If we used `@append_spaced_softline` rather than
`@append_spaced_scoped_softline`, the `1,` would be followed by a space
rather than a newline, because it's inside a single-line
`product_expression` node.

### Testing context with predicates

Sometimes, similarly to what happens with scoped softlines, we want a
query to match only if the scoped context is single-line, or multi-line.
Topiary has several predicates that achieve this result.

#### `#single_line_scope_only!` / `#multi_line_scope_only!`

These predicates allow the query to trigger only if the associated custom scope
containing the matched nodes are is single-line (resp. multi-line).

> **Note**\
> There are non-scoped equivalents to these predicates; please see
> [vertical spacing](vertical-spacing.md#testing-context-with-predicates)
> for details.

##### Example

```scheme
; Allow (and enforce) the optional "|" before the first match case
; in function expressions in OCaml if and only if the scope is multi-line
(function_expression
  (match_case)? @do_nothing
  .
  "|" @delete
  .
  (match_case)

  (#single_line_scope_only! "function_definition")
)

(function_expression
  "|"? @do_nothing
  .
  (match_case) @prepend_delimiter

  (#multi_line_scope_only! "function_definition")
  (#delimiter! "| ") ; sic
)
```

## Measuring scopes

### `@append_begin_measuring_scope` / `@prepend_begin_measuring_scope` / `@append_end_measuring_scope` / `@prepend_end_measuring_scope`

Sometimes, custom scopes are not enough: we may want to format a node
depending on the multi-line-ness of a piece of code that does not
include the node in question. For instance, consider this function
application in OCaml:

```ocaml
foo bar (fun x -> qux)
```

We may want to format it as either of the following two, depending on
the actual length of `foo`, `bar`, and `qux`:

```ocaml
foo bar (fun x ->
  qux
)
```
```ocaml
foo
  bar
  (fun x ->
    qux
  )
```

If `foo bar` is single-line, we don't want to wrap `(fun x -> qux)` onto
a new line. However, if `foo bar` is multi-line, then we do, to be
consistent with `bar`.

Because custom scopes can only impact the behaviour of nodes inside the
scope, we can't use them to solve this issue. This is why we need
"measuring scopes".

Measuring scopes are opened/closed with a similar syntax as "regular"
custom scopes, with any of the following capture names, in conjunction
with the `#scope_id!` predicate:

- `@append_begin_measuring_scope` / `@prepend_begin_measuring_scope`
- `@append_end_measuring_scope` / `@prepend_end_measuring_scope`

Measuring scopes behave as follows:

- A measuring scope must always be contained in a regular custom scope
  with the same `#scope_id!`. There can't be two measuring scopes with
  the same `#scope_id!` inside the same regular custom scope.

- If a regular custom scope contains a measuring scope, then all tags
  contained in the regular scope that depend on its multi-line-ness will
  instead depend on the multi-line-ness of the measuring scope (hence
  the name: the inner, measuring scope measures the multi-line-ness of
  the outer, regular scope).

#### Example

The example below solves the problem of indenting function application
in OCaml stated above, using measuring scopes.

```scheme
(application_expression
  .
  (_) @append_indent_start @prepend_begin_scope @prepend_begin_measuring_scope
  (#scope_id! "function_application")
  (_) @append_end_scope
  .
)

; The end of the measuring scope depends on the last argument: if it's a function,
; end it before the function, otherwise end it after the last argument. In that case,
; it's the same as the regular scope.
(application_expression
  (#scope_id! "function_application")
  (_
    [
      (fun_expression)
      (function_expression)
    ]? @do_nothing
  ) @append_end_measuring_scope
  .
)

(application_expression
  (#scope_id! "function_application")
  (_
    [
      (fun_expression)
      (function_expression)
    ] @prepend_end_measuring_scope
  )
  .
)

; If the measuring scope is single-line, end indentation _before_ the last node.
; Otherwise, end the indentation after the last node.
(application_expression
  (#multi_line_scope_only! "function_application")
  (_) @append_indent_end
  .
)

(application_expression
  (#single_line_scope_only! "function_application")
  (_) @prepend_indent_end
  .
)
```

## `@append_empty_scoped_softline` / `@prepend_empty_scoped_softline`

The matched nodes will have an empty softline appended (or,
respectively, prepended) to them. This will be expanded to a line break
for multi-line nodes within the scope defined by the `#scope_id!`
predicate and to nothing for single-line nodes.

### Example

```scheme
; Define a scope immediately following a command up to (but not
; including) the next node. If that scope is multi-line, then insert a
; line break after the command; otherwise, insert nothing.
(program
  (command) @append_begin_scope @append_empty_scoped_softline
  .
  _ @prepend_end_scope

  (#scope_id! "program_line_break")
)
```

## `@append_spaced_scoped_softline` / `@prepend_spaced_scoped_softline`

The matched nodes will have a spaced softline appended (or,
respectively, prepended) to them. This will be expanded to a line break
for multi-line nodes within the scope defined by the `#scope_id!`
predicate and to a space for single-line nodes.

### Example

```scheme
; Define a scope after the equals sign in a let binding that's followed
; by a function expression. If that scope is multi-line, then insert a
; line break after the function arrow; otherwise, insert a space.
(let_binding
  "=" @prepend_begin_scope
  .
  (fun_expression
    "->" @append_spaced_scoped_softline
  ) @append_end_scope

  (#scope_id! "fun_definition")
)
```
