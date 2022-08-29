# Tree-sitter experiment

This project attempts to answer the question if it is possible to create a
uniform formatter for simple languages using the tree-sitter ecosystem.

## Example

The program can be run like this:

```
echo '{"foo":"bar"}' | tree-sitter-formatter --language json
```

Or like this:

```
echo '{"foo":"bar"}' | cargo run -- --language json
```

It will output the following formatted code:

```
{
    "foo": "bar"
}
```

## Design

Tree-sitter will provide the tool with the concrete syntax tree of
the language. The tool will also parse a set of queries created specifically
for it to inform it how to format the provided CST. The tool will do the following.

1. Create a flat list of terminal nodes from the CST. It removes all whitespace nodes or anonymous extra nodes (TBD).
2. Run the queries with the tree-sitter language and provided source file.
3. Update the flat list as defined by the queries.
4. Print the flat list.

It might be useful to convert the flat list to a list of documents from the `pretty` crate before outputting, but that can be done later.
