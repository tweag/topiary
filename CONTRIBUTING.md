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
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -- -l ocaml < topiary/tests/samples/input/ocaml.ml > formatted.ml
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
