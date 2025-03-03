# Adding a new language

This section illustrates how to add a supported language to Topiary, provided it already has a tree-sitter grammar.

We will use C as the running example in this section.

### Minimal steps

The two following steps are enough to jumpstart the formatting of a new language:

#### Register the grammar in `topiary-config/languages.ncl`:

```nickel
    c = {
      extensions = ["c", "h"],
      grammar.source.git = {
        git = "https://github.com/tree-sitter/tree-sitter-c.git",
        rev = "6c7f459ddc0bcf78b615d3a3f4e8fed87b8b3b1b",
      },
    },
```

#### Create the query file
```bash
touch topiary-queries/queries/c.scm
```

#### Testing

You can now check that Topiary is able to "format" your new language with:

```bash
$ echo 'void main();' | cargo run -- format -s --language c
voidmain();
```

```bash
$ echo 'void main();' > foo.c && cargo run -- format -s foo.c && cat foo.c
voidmain();
```

### Add the new language to the test suite

#### Create input/expected files
```bash
echo 'void main ();' > topiary-cli/tests/samples/input/c.c
echo 'voidmain();' > topiary-cli/tests/samples/expected/c.c
```

#### Add the Cargo feature flags

##### In `topiary-cli/Cargo.toml`
```toml
experimental = [
  "clang",
]

clang = ["topiary-config/clang", "topiary-queries/clang"]
```

##### In `topiary-config/Cargo.toml`
```toml
clang = []

all = [
  "clang",
]
```

##### In `topiary-queries/Cargo.toml`
```toml
clang = []
```

#### Add tests in `topiary-cli/tests/sample-tester.rs`

<!-- FIXME Change this to use new macros -->

```rust
fn input_output_tester() {

[...]

    #[cfg(feature = "clang")]
    io_test("c.c");

[...]

fn coverage_tester() {

[...]

    #[cfg(feature = "clang")]
    coverage_test("c.c");
```

#### Testing
You should be able to successfully run the new tests with
```bash
cargo test --no-default-features -F clang -p topiary-cli --test sample-tester
```

### Include the query file in Topiary at compile time

#### In `topiary-queries/src/lib.rs`
```rust
/// Returns the Topiary-compatible query file for C.
#[cfg(feature = "clang")]
pub fn c() -> &'static str {
    include_str!("../queries/c.scm")
}
```

#### In `topiary-cli/src/io.rs`
```rust
fn to_query<T>(name: T) -> CLIResult<QuerySource>

[...]

        #[cfg(feature = "clang")]
        "c" => Ok(topiary_queries::c().into()),
```

This will allow your query file to by considered as the default fallback query, when no other file can be found at runtime for your language.
