# Contributing to Topiary

<!-- TODO: Nix devshell -->

## Performance

You can check performance before or after changes by running `cargo bench`.

If you do `cargo install flamegraph`, you can generate a performance flamegraph
like this:

```bash
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -- -l ocaml < topiary-cli/tests/samples/input/ocaml.ml > formatted.ml
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
that is provided by Nix, if you have both installed.

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

If you need to add or update Tree-sitter grammar Wasm files, the easiest way would be using Nix.

Simply enter our `devShell` with `nix develop`, and then run `update-wasm-grammars`.
Alternatively, if you have `git`, `tree-sitter` and `emcc` in your `PATH`, you can run the `./update-wasm-grammars.sh` file.

To use docker instead, the legacy approach can still be used (using JSON as an example):

1. Make sure you have Docker running and that you are member of the `docker`
   group so you can run it without being root.
2. `npm install tree-sitter-cli` (or some other way)
3. `npm install tree-sitter-json` (or by cloning the git repository)
   - If you used npm, tree-sitter-json will be fetched under `node_modules/tree-sitter-json/`
   - If you used git, it will be wherever you cloned the repository (most likely `tree-sitter-json/`)

   Whichever of these options you pick, we will assume `JSON_GRAMMAR` is the directory where the `grammar.js` can be found.
4. Make sure you have a file at
   `JSON_GRAMMAR/src/grammar.json`.
5. Run `npx tree-sitter build-wasm JSON_GRAMMAR`. If you get a Docker permission
   error, you may have to add yourself to the docker group.
6. `mv tree-sitter-json.wasm web-playground/public/scripts/`

For OCaml, the process is slightly different because the tree-sitter-ocaml repository/package contains two grammars:

1. `npm install tree-sitter-cli`
2. `npm install tree-sitter-ocaml` (or git, like above)
3. Run `npx tree-sitter build-wasm OCAML_GRAMMAR/ocaml`.
4. Run `npx tree-sitter build-wasm OCAML_GRAMMAR/ocaml_interface`.
5. `mv tree-sitter-ocaml*.wasm web-playground/public/scripts/`

The playground frontend is a small React app. You can run a development server
for that like this:

```bash
cd web-playground
npm install
npm run dev
```

If you want to build the playground so it works with the full website running
with Deno as above, you can now just do `npm run build`.
