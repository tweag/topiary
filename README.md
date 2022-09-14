# Tree-sitter experiment

This project attempts to answer the question if it is possible to create a
uniform formatter for simple languages using the tree-sitter ecosystem.

## Example

The program can be run like this:

```bash
echo '{"foo":"bar"}' | tree-sitter-formatter --language json
```

Or, if you have Rust or Nix installed, like this:

```bash
echo '{"foo":"bar"}' | cargo run -- --language json
echo '{"foo":"bar"}' | nix run . -- --language json
```

It will output the following formatted code:

```json
{
    "foo": "bar"
}
```

Insert `RUST_LOG=debug` in front of `tree-sitter-formatter` (or `cargo` or `nix`) if you want to enable debug logging.

## Design

Tree-sitter will provide the tool with the concrete syntax tree of
the language. The tool will also parse a set of queries created specifically
for it to inform it how to format the provided CST. The tool will do the following.

1. Create a flat list of terminal nodes from the CST. It removes all whitespace nodes or anonymous extra nodes (TBD).
2. Run the queries with the tree-sitter language and provided source file.
3. Update the flat list as defined by the queries.
4. Print the flat list.

It might be useful to convert the flat list to a list of documents from the `pretty` crate before outputting, but that can be done later.

## Contributing

Issues and pull requests are welcome! If you have Nix installed, you can start a development shell with Rust like this:

```bash
nix develop
```

### Performance

You can check performance before or after changes by running `cargo bench`.

If you do `cargo install flamegraph`, you can generate a performance flamegraph like this:

```bash
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -- -l ocaml < tests/samples/input/ocaml.ml > formatted.ml
```
