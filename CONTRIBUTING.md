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

## Code Coverage

Code coverage metrics can be generated via LLVM profiling data generated
by the build. These can be created by setting appropriate environment
variables to `cargo test`:

```bash
CARGO_INCREMENTAL=0 \
RUSTFLAGS='-Cinstrument-coverage' \
LLVM_PROFILE_FILE='cargo-test-%p-%m.profraw' \
cargo test
```

This will build and run the test suite and output
`cargo-test-*-*.profraw` files in the working directory. (Outside of the
Nix development shell, you may need `binutils` installed.)

These files can be used by [`grcov`](https://github.com/mozilla/grcov)
to render a variety of output reports. For example, the following
renders HTML output in `target/coverage/html`:

```bash
grcov --branch \
      --output-type html \
      --source-dir src \
      --binary-path target/debug/deps \
      --output-path target/coverage/html \
      .
```

:warning: `grcov` relies on the `llvm-tools-preview` component for
`rustup`. For Nix users, `rustup` can interfere with the Rust toolchain
that is provided by Nix, if you have both installed. For convenience,
the `generate-coverage.sh` script can be run from the root of this
repository to avoid contaminating your environment, but note it will
download a full toolchain on each run.

## Web site and web playground

If you have [Deno](https://deno.land/) installed, you can start a local web
server like this:

```bash
deno run -A local-web-server.ts
```

The web site should then be running on http://localhost:8080.

In order to build or update the Wasm playground, you can run this:

```bash
./update-wasm-app.sh
```

If you need to add or update Tree-sitter grammar Wasm files, you can do it like
this (using JSON as an example):

1. Make sure you have Docker running.
2. npm install --save-dev tree-sitter-cli tree-sitter-json
3. npm install --save-dev tree-sitter-json
4. Alternatively, clone a Git repo with the grammars and copy it into `node_modules`.
5. Make sure you have a file at
   `node_modules/tree-sitter-json/src/grammar.json`. In case of OCaml, you have
   to copy some directories (you also have to move a `common` directory).
6. npx tree-sitter build-wasm node_modules/tree-sitter-json
7. mv tree-sitter-json.wasm website/playground/scripts/

The playground frontend is a small React app. You can run a development server for that like this:

```bash
cd web-playground/react-app
npm start
```

If you want to build the playground so it works with the full website running with Deno as above,
you can do:

```bash
cd web-playground/react-app
npm run build
```
