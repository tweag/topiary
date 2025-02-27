# Basics

<!----------------------------------------------------------------------
FIXME? "Basics" is not the best title here; it's more like
"Miscellaneous stuff that didn't fit anywhere else". It could be made
into a "Basics" section, by overviewing the query language, but the
Tree-sitter documentation already does a good job of this.

Suggestions...
----------------------------------------------------------------------->

## `@leaf`

Some nodes should not have their contents formatted at all; the classic
example being string literals. The `@leaf` capture will mark such nodes
as leaves -- even if they admit their own structure -- and leave them
unformatted.

### Example

```scheme
; Don't format strings or comments
[
  (string)
  (comment)
] @leaf
```
## A note on anchors

The behaviour of "anchors" can be counter-intuitive. Consider, for
instance, the following query:

```scheme
(
  (list_entry) @append_space
  .
)
```

One might assume that this query only matches the final element in the
list but this is not true. Since we did not explicitly march a parent
node, the engine will match on every `list_entry`. After all, the when
looking only at the nodes in the query, the `list_entry` is indeed the
last node.

To resolve this issue, match explicitly on the parent node:

```scheme
(list
  (list_entry) @append_space
  .
)
```

Or even implicitly:

```scheme
(_
  (list_entry) @append_space
  .
)
```

Note that a capture is put after the node it is associated with. If you
want to put a space in front of a node, you do it like this:

```scheme
(infix_operator) @prepend_space
```

This, on the other hand, will not work:

```scheme
@append_space (infix_operator)
```

## `@do_nothing`

If any of the captures in a query match are `@do_nothing`, then the
match will be ignored.

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

## `#query_name!`

When the logging verbosity is set to `-vv` or higher, Topiary outputs information about which queries are matched, for instance:
```
[2024-10-08T15:48:13Z INFO  topiary_core::tree_sitter] Processing match: LocalQueryMatch { pattern_index: 17, captures: [ {Node "," (1,3) - (1,4)} ] } at location (286,1)
```
The predicate `#query_name!` takes a string argument, is optional, and can be added to any query.
It will modify the log line to display its argument.

### Example

Considering the log line above, and let us assume that the query at `location (286,1)` is:

```scheme
(
  "," @append_space
  .
  (_)
)
```

If we add a `query_name` predicate:

```scheme
(
  (#query_name! "comma spacing")
  "," @append_space
  .
  (_)
)
```

Then the log line will become:

```
[2024-10-08T15:48:13Z INFO  topiary_core::tree_sitter] Processing match of query "comma spacing": LocalQueryMatch { pattern_index: 17, captures: [ {Node "," (1,3) - (1,4)} ] } at location (286,1)
```

## Query and capture precedence

Formatting is not necessarily invariant over the order of queries. For
example, queries that add delimiters or remove nodes can have a
different effect on the formatted output depending on the order in which
they appear in the query file.

Consider, say, the following two queries for the Bash grammar:

```scheme
; Query A: Append semicolon
(
  (word) @append_delimiter
  .
  ";"? @do_nothing

  (#delimiter! ";")
)

; Query B: Surround with quotes
(
  "\""? @do_nothing
  .
  (word) @prepend_delimiter @append_delimiter
  .
  "\""? @do_nothing

  (#delimiter! "\"")
)
```

In the order presented above (`A`, then `B`), then the input `foo` will
be formatted as:

```
"foo;"
```

In the opposite order (`B`, then `A`), Topiary will however produce the
following output:

```
"foo";
```

A similar consideration exists for captures. That is, while most
captures do not meaningfully affect one another, there are three notable
exceptions:

1. `@do_nothing` will cancel all other captures in a matched query. This
   takes the highest priority.

2. `@delete` will delete any matched node, providing the matching query
   is not cancelled.

3. `@leaf` will suppress formatting within that node, even if it admits
   some internal structure. However, leaf nodes are still subject to
   deletion.

<!----------------------------------------------------------------------
TODO: Other predicates supported by Tree-sitter (e.g., #match, etc.)
----------------------------------------------------------------------->
