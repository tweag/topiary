# Capture names

As long as there is a [Tree-sitter grammar][tree-sitter:parsers] defined
for a language, Tree-sitter can parse it and provide a concrete syntax
tree (CST). Tree-sitter will also allow us to run queries against this
tree. We can make use of that to define how a language should be
formatted, by annotating nodes with "capture names" that define
formatting directives.

This chapter assumes you are already familiar with the [Tree-sitter
query language][tree-sitter:query] and documents all the capture names
recognised by Topiary to drive formatting.

## Example

> **Note**\
> This example is derived from the [Topiary announcement blog
> post][tweag:topiary-announcement]. Please see the post for additional
> detail.

```scheme
(
  [
    (infix_operator)
    "if"
    ":"
  ] @append_space
  .
  (_)
)
```

This will match any node that the grammar has identified as an
`infix_operator`, or the anonymous nodes containing `if` or `:` tokens,
immediately followed by any named node (represented by the `(_)`
wildcard pattern). The query matches on subtrees of the same shape,
where the annotated node within it will be "captured" with the name
`@append_space`. Topiary runs through all matches and captures, and when
we process any capture called `@append_space`, we append a space after
the annotated node.

Before rendering the output, Topiary does some post-processing, such as
squashing consecutive spaces and newlines, trimming extraneous
whitespace, and ordering indentation and newline instructions
consistently. This means that you can, for example, prepend and append
spaces to `if` and `true`, and Topiary will still output `if true` with
just one space between the words.

<!-- Links -->
[tree-sitter:parsers]: https://github.com/tree-sitter/tree-sitter/wiki/List-of-parsers
[tree-sitter:query]: https://tree-sitter.github.io/tree-sitter/using-parsers/queries/index.html
[tweag:topiary-announcement]: https://www.tweag.io/blog/2023-03-09-announcing-topiary
