# Capture names

As long as there is a [Tree-sitter grammar][tree-sitter-parsers] defined
for a language, Tree-sitter can parse it and provide a concrete syntax
tree (CST). Tree-sitter will also allow us to run queries against this
tree. We can make use of that to define how a language should be
formatted, by annotating nodes with "capture names" that define
formatting directives.

This chapter documents all the capture names recognised by Topiary to
drive formatting.

## Example

```scheme
[
  (infix_operator)
  "if"
  ":"
] @append_space
```

<!-- TODO See also https://www.tweag.io/blog/2023-03-09-announcing-topiary -->
This query will match any node that the grammar has identified to be an
`infix_operator`, as well as any anonymous node containing `if` or `:`.
The match will be captured with the name `@append_space`. Topiary runs
through all matches and captures, and when we process any capture called
`@append_space`, we will append a space after the matched node.

<!-- TODO Move this paragraph to vertical spacing chapter -->
The formatter goes through the CST nodes and detects all that are
spanning more than one line. This is interpreted to be an indication
from the programmer who wrote the input that the node in question should
be formatted as multi-line. Any other nodes will be formatted as
single-line. Whenever a query match has inserted a _softline_, it will
be expanded to a newline if the node is multi-line, or to a space or
nothing if the node is single-line, depending on whether
`@append_spaced_softline` or `@append_empty_softline` was used.

The aggregation of these capture names on appropriate queries, when run
through Topiary, amounts to a formatting style for the language
described by that grammar.

Before rendering the output, the formatter will do a number of clean up
operations, such as reducing consecutive spaces and newlines to one,
trimming spaces at end of lines and leading and trailing blanks lines,
and ordering indenting and newline instructions consistently.

This means that you can for example prepend and append spaces to `if`
and `true`, and we will still output `if true` with just one space
between the words.

<!-- Links -->
[tree-sitter-parsers]: https://tree-sitter.github.io/tree-sitter/#available-parsers
