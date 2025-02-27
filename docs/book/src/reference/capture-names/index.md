# Capture names

As long as there is a [Tree-sitter grammar][tree-sitter-parsers] defined
for a language, Tree-sitter can parse it and provide a concrete syntax
tree (CST). Tree-sitter will also allow us to run queries against this
tree. We can make use of that to define how a language should be
formatted, by annotating nodes with "capture names" that define
formatting directives.

This chapter assumes you are already familiar with the [Tree-sitter
query language][tree-sitter-query] and documents all the capture names
recognised by Topiary to drive formatting.

## Example

```scheme
[
  (infix_operator)
  "if"
  ":"
] @append_space
```

<!----------------------------------------------------------------------
TODO: This isn't a great fit here. Rework it bit.

See also https://www.tweag.io/blog/2023-03-09-announcing-topiary
----------------------------------------------------------------------->

This query will match any node that the grammar has identified to be an
`infix_operator`, as well as any anonymous node containing `if` or `:`.
The match will be captured with the name `@append_space`. Topiary runs
through all matches and captures, and when we process any capture called
`@append_space`, we will append a space after the matched node.

The aggregation of these captures on appropriate queries, when run
through Topiary, amounts to a formatting style for the language
described by that grammar. Then, before rendering the final output,
Topiary will do a number of clean up operations, such as reducing
consecutive spaces and newlines to one, trimming spaces at end of lines
and leading and trailing blanks lines, and ordering indenting and
newline instructions consistently.

This means that you can for example prepend and append spaces to `if`
and `true`, and we will still output `if true` with just one space
between the words.

<!-- Links -->
[tree-sitter-parsers]: https://github.com/tree-sitter/tree-sitter/wiki/List-of-parsers
[tree-sitter-query]: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/index.html
