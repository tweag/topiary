# Topiary

[![Latest Release][badge-release]][badge-release-link]
[![CI Status][badge-ci]][badge-ci-link]
[![Discord][badge-discord]][badge-discord-link]

* [Topiary web site][topiary-website]
* [Topiary playground][topiary-playground]

## Add a new language

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

## Suggested workflow

In order to work productively on query files, the following is one
suggested way to work:

1. If you're working on a new language, follow the steps in [the previous section](#add-a-new-language).

2. Add a snippet of code you want to format to `topiary-cli/tests/samples/input/mylanguage.mlg`.

3. Add the properly formatted version of the code to `topiary-cli/tests/samples/expected/mylanguage.mlg`.

4. Run:

   ```bash
   cargo test \
     --no-default-features \
     -F mylanguage \
     -p topiary-cli \
     input_output_tester \
     -- --nocapture
   ```

   Provided it works, it should output a _lot_ of log messages. Copy
   that output to a text editor. You are particularly interested in the
   CST output that starts with a line like this: `CST node: {Node
   compilation_unit (0, 0) - (5942, 0)} - Named: true`.

> [!TIP]
> As an alternative to using the debugging output, the `vis`
> visualisation subcommand line option exists to output the Tree-sitter
> syntax tree in a variety of formats.

5. The test run will output all the differences between the actual
   output and the expected output, e.g. missing spaces between tokens.
   Pick a difference you would like to fix, and find the line number and
   column in the input file.

> [!NOTE]
> Keep in mind that the CST output uses 0-based line and column numbers,
> so if your editor reports line 40, column 37, you probably want line
> 39, column 36.

6. In the CST debug or visualisation output, find the nodes in this
   region, such as the following:

   ```
   [DEBUG atom_collection] CST node:   {Node constructed_type (39, 15) - (39, 42)} - Named: true
   [DEBUG atom_collection] CST node:     {Node type_constructor_path (39, 15) - (39, 35)} - Named: true
   [DEBUG atom_collection] CST node:       {Node type_constructor (39, 15) - (39, 35)} - Named: true
   [DEBUG atom_collection] CST node:     {Node type_constructor_path (39, 36) - (39, 42)} - Named: true
   [DEBUG atom_collection] CST node:       {Node type_constructor (39, 36) - (39, 42)} - Named: true
   ```

7. This may indicate that you would like spaces after all
   `type_constructor_path` nodes:

   ```scheme
   (type_constructor_path) @append_space
   ```

   Or, more likely, you just want spaces between pairs of them:

   ```scheme
   (
     (type_constructor_path) @append_space
     .
     (type_constructor_path)
   )
   ```

   Or maybe you want spaces between all children of `constructed_type`:

   ```scheme
   (constructed_type
     (_) @append_space
     .
     (_)
   )
   ```

8. Run `cargo test` again, to see if the output is better now, and then
   return to step 5.

<!-- Links -->

[badge-ci]: https://img.shields.io/github/actions/workflow/status/tweag/topiary/ci.yml?logo=github
[badge-ci-link]: https://github.com/tweag/topiary/actions/workflows/ci.yml
[badge-discord]: https://img.shields.io/discord/1174731094726295632?logo=discord
[badge-discord-link]: https://discord.gg/FSnkvNyyzC
[badge-release]: https://img.shields.io/github/v/release/tweag/topiary?display_name=release&logo=github
[badge-release-link]: https://github.com/tweag/topiary/releases/latest
[topiary-playground]: https://topiary.tweag.io/playground
[topiary-website]: https://topiary.tweag.io
