# Contributing

Issues and pull requests are welcome! If you have Nix installed, you can start a
development shell with Rust like this:

```bash
nix develop
```

## Performance

You can check performance before or after changes by running `cargo bench`.

If you do `cargo install flamegraph`, you can generate a performance flamegraph
like this:

```bash
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -- -l ocaml < tests/samples/input/ocaml.ml > formatted.ml
```
