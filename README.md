# Tree-sitter experiment

This project attempts to answer the question if it is possible to create a
uniform formatter for simple languages using the tree-sitter ecosystem.

## Design
Tree-sitter will provide the tool with the concrete syntax tree of
the language. The tool will also parser a set of queries created specifically
for it to inform it how to format the provided CST. The tool will do the following.

1. Create a flat list of terminal nodes from the CST. It removes all whitespace nodes or anonymous extra nodes (TBD).
1. Run the queries with the tree-sitter language and provided source file.
1. Update the flat list as defined by the queries.
1. Print the flat list.

It might be useful to convert the flat list to a list of documents from the `pretty` crate before outputting, but that can be done later.
