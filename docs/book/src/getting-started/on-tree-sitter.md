# Tree-sitter and its queries

From the [Tree-sitter documentation][tree-sitter:intro]:

> Tree-sitter is a parser generator tool and an incremental parsing
> library. It can build a concrete syntax tree for a source file and
> efficiently update the syntax tree as the source file is edited.

Any non-trivial formatter needs to work with code's syntax tree. Topiary
uses Tree-sitter to this end, which precludes the need to write a new
parser each time

[tree-sitter:intro]: https://tree-sitter.github.io/tree-sitter
[tree-sitter:known-grammars]: https://github.com/tree-sitter/tree-sitter/wiki/List-of-parsers

<!----------------------------------------------------------------------
TODO: Give an explanation/overview of Tree-sitter query files and how
Topiary uses them.

For a subsequent PR...

(See https://github.com/tweag/topiary/pull/750/files#r1961950658)
----------------------------------------------------------------------->
