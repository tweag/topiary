# Adding a new language

This chapter illustrates how to add a supported language to Topiary,
provided it already has a Tree-sitter grammar.

We will use C as the running example. The following steps are enough to
bootstrap the formatting of a new language.

## Register the grammar in `topiary-config/languages.ncl`:

```nickel
    clang = {
      extensions = ["c", "h"],
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-c.git",
        rev = "6c7f459ddc0bcf78b615d3a3f4e8fed87b8b3b1b",
      },
    },
```

## Create the query file

```sh
touch topiary-queries/queries/clang.scm
```

### Testing

You can now check that Topiary is able to "format" your new language
with:

```sh
$ echo 'void main();' | cargo run -- format -s --language clang
voidmain();
```

At this point, there are no formatting queries, so no formatting is
applied and we get mangled output. This is why the `-s`
(`--skip-idempotence`) flag is set, as Topiary will complain when
formatting doesn't reach a fixed point (which this output is likely to
result in).

## Add the new language to the test suite

### Create input/expected files

Topiary's I/O tester will format input files and check them against an
expected output. For the time being, let's stick with the mangled
output, so we can get the tests to run and pass.

```sh
echo 'void main ();' > topiary-cli/tests/samples/input/clang.c
echo 'voidmain();' > topiary-cli/tests/samples/expected/clang.c
```

### Add Cargo feature flags

Each language is gated behind a feature flag. We'll use the `clang`
feature flag for C -- to match the language name in the configuration --
which needs to be added in the appropriate places.

#### In `topiary-cli/Cargo.toml`

```toml
experimental = [
  "clang",
]

clang = ["topiary-config/clang", "topiary-queries/clang"]
```

#### In `topiary-config/Cargo.toml`

```toml
clang = []

all = [
  "clang",
]
```

#### In `topiary-queries/Cargo.toml`

```toml
clang = []
```

### Add tests in `topiary-cli/tests/sample-tester.rs`

To register the I/O and coverage tests for the new language, we need to
add it to the test suite.

#### In `fn get_file_extension`

You will need to add a mapping from the language (feature name) to the
file extension under test:

```rust
fn get_file_extension(language: &str) -> &str {
    match language {

        [...]

        "clang" => "c",

        [...]

    }
}
```

#### In `mod test_fmt`

Then you'll need to add the language to the `lang_test!` macro calls in
the `test_fmt` module, respectively:

```rust
    lang_test!(

        [...]

        "clang",

        [...]

        fmt_input
    );
```

#### In `mod test_coverage`

Likewise in the `test_coverage` module:

```rust
    lang_test!(

        [...]

        "clang",

        [...]

        coverage_input
    );
```

### Testing

You should be able to successfully run the new tests with:

```sh
cargo test --no-default-features -F clang -p topiary-cli --test sample-tester
```

## Include the query file in Topiary at compile time

### In `topiary-queries/src/lib.rs`

```rust
/// Returns the Topiary-compatible query file for C.
#[cfg(feature = "clang")]
pub fn clang() -> &'static str {
    include_str!("../queries/clang.scm")
}
```

### In `topiary-cli/src/io.rs`

```rust
fn to_query<T>(name: T) -> CLIResult<QuerySource>

[...]

        #[cfg(feature = "clang")]
        "clang" => Ok(topiary_queries::clang().into()),
```

This will allow your query file to by considered as the default fallback
query, when no other file can be found at runtime for your language.

## Iterate

Once the above steps have been completed, Topiary will be able to use
the C Tree-sitter grammar and the formatting queries in `clang.scm` to
format C code.

You can now iterate on the formatting queries and the respective input
and expected sample files to build your formatter, using the I/O and
coverage tests to guide the process. Please also see the [following
chapter](suggested-workflow.md) on query development for more
information.
