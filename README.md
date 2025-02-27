# Topiary

[![Latest Release][badge-release]][badge-release-link]
[![CI Status][badge-ci]][badge-ci-link]
[![Discord][badge-discord]][badge-discord-link]

* [Topiary web site][topiary-website]
* [Topiary playground][topiary-playground]


## Supported capture instructions

### `@allow_blank_line_before`

The matched nodes will be allowed to have a blank line before them, if
specified in the input. For any other nodes, blank lines will be
removed.

#### Example

```scheme
; Allow comments and type definitions to have a blank line above them
[
  (comment)
  (type_definition)
] @allow_blank_line_before
```

### `@append_delimiter` / `@prepend_delimiter`

The matched nodes will have a delimiter appended to them. The delimiter
must be specified using the predicate `#delimiter!`.

#### Example

```scheme
; Put a semicolon delimiter after field declarations, unless they already have
; one, in which case we do nothing.
(
  (field_declaration) @append_delimiter
  .
  ";"* @do_nothing
  (#delimiter! ";")
)
```

If there is already a semicolon, the `@do_nothing` instruction will be
activated and prevent the other instructions in the query (the
`@append_delimiter`, here) from applying. Otherwise, the `";"*` captures
nothing and in this case the associated instruction (`@do_nothing`) does
not activate.

Note that `@append_delimiter` is the same as `@append_space` when the
delimiter is set to `" "` (i.e., a space).

### `@append_empty_softline` / `@prepend_empty_softline`

The matched nodes will have an empty softline appended or prepended to
them. This will be expanded to a newline for multi-line nodes and to
nothing for single-line nodes.

#### Example

```scheme
; Put an empty softline before dots, so that in multi-line constructs we start
; new lines for each dot.
(_
  "." @prepend_empty_softline
)
```

### `@append_hardline` / `@prepend_hardline`

The matched nodes will have a newline appended or prepended to them.

#### Example

```scheme
; Consecutive definitions must be separated by line breaks
(
  (value_definition) @append_hardline
  .
  (value_definition)
)
```

### `@append_indent_start` / `@prepend_indent_start`

The matched nodes will trigger indentation before or after them. This
will only apply to lines following, until an indentation end is
signalled. If indentation is started and ended on the same line, nothing
will happen. This is useful, because we get the correct behaviour
whether a node is formatted as single-line or multi-line. It is
important that all indentation starts and ends are balanced.

#### Example

```scheme
; Start an indented block after these
[
  "begin"
  "else"
  "then"
  "{"
] @append_indent_start
```

### `@append_indent_end` / `@prepend_indent_end`

The matched nodes will trigger that indentation ends before or after
them.

#### Example

```scheme
; End the indented block before these
[
  "end"
  "}"
] @prepend_indent_end

; End the indented block after these
[
  (else_clause)
  (then_clause)
] @append_indent_end
```

### `@append_input_softline` / `@prepend_input_softline`

The matched nodes will have an input softline appended or prepended to
them. An input softline is a newline if the node has a newline in front
of it in the input document, otherwise it is a space.

#### Example

```scheme
; Input softlines before and after all comments. This means that the input
; decides if a comment should have line breaks before or after. But don't put a
; softline directly in front of commas or semicolons.

(comment) @prepend_input_softline

(
  (comment) @append_input_softline
  .
  [ "," ";" ]* @do_nothing
)
```

### `@append_spaced_softline` / `@prepend_spaced_softline`

The matched nodes will have a spaced softline appended or prepended to
them. This will be expanded to a newline for multi-line nodes and to a
space for single-line nodes.

#### Example

```scheme
; Append spaced softlines, unless there is a comment following.
(
  [
    "begin"
    "else"
    "then"
    "->"
    "{"
    ";"
  ] @append_spaced_softline
  .
  (comment)* @do_nothing
)
```

### `@delete`

Remove the matched node from the output.

#### Example

```scheme
; Move semicolon after comments.
(
  ";" @delete
  .
  (comment)+ @append_delimiter
  (#delimiter! ";")
)
```


### `@multi_line_indent_all`

To be used on comments or other leaf nodes, to indicate that we should indent
all its lines, not just the first.

#### Example

```scheme
(#language! ocaml)
(comment) @multi_line_indent_all
```

### `@single_line_no_indent`

The matched node will be printed alone, on a single line, with no indentation.

#### Example

```scheme
(#language! ocaml)
; line number directives must be alone on their line, and can't be indented
(line_number_directive) @single_line_no_indent
```

### Understanding the different newline captures

| Type            | Single-Line Context | Multi-Line Context |
| :-------------- | :------------------ | :----------------- |
| Hardline        | Newline             | Newline            |
| Empty Softline  | Nothing             | Newline            |
| Spaced Softline | Space               | Newline            |
| Input Softline  | Input-Dependent     | Input-Dependent    |

"Input softlines" are rendered as newlines whenever the targeted node
follows a newline in the input. Otherwise, they are rendered as spaces.

#### Example

Consider the following JSON, which has been hand-formatted to exhibit
every context under which the different newline capture names operate:

```json
{
  "single-line": [1, 2, 3, 4],
  "multi-line": [
    1, 2,
    3
    , 4
  ]
}
```

We'll apply a simplified set of JSON format queries that:
1. Opens (and closes) an indented block for objects;
2. Each key-value pair gets its own line, with the value split onto a
   second;
3. Applies the different newline capture name on array delimiters.

That is, iterating over each `@NEWLINE` type, we apply the following:

```scheme
(#language! json)

(object . "{" @append_hardline @append_indent_start)
(object "}" @prepend_hardline @prepend_indent_end .)
(object (pair) @prepend_hardline)
(pair . _ ":" @append_hardline)

(array "," @NEWLINE)
```

The first two formatting rules are just for clarity's sake. The last
rule is what's important; the results of which are demonstrated below:

##### `@append_hardline`

```json
{
  "single-line":
  [1,
  2,
  3,
  4],
  "multi-line":
  [1,
  2,
  3,
  4]
}
```

##### `@prepend_hardline`

```json
{
  "single-line":
  [1
  ,2
  ,3
  ,4],
  "multi-line":
  [1
  ,2
  ,3
  ,4]
}
```

##### `@append_empty_softline`

```json
{
  "single-line":
  [1,2,3,4],
  "multi-line":
  [1,
  2,
  3,
  4]
}
```

##### `@prepend_empty_softline`

```json
{
  "single-line":
  [1,2,3,4],
  "multi-line":
  [1
  ,2
  ,3
  ,4]
}
```

##### `@append_spaced_softline`

```json
{
  "single-line":
  [1, 2, 3, 4],
  "multi-line":
  [1,
  2,
  3,
  4]
}
```

##### `@prepend_spaced_softline`

```json
{
  "single-line":
  [1 ,2 ,3 ,4],
  "multi-line":
  [1
  ,2
  ,3
  ,4]
}
```

##### `@append_input_softline`

```json
{
  "single-line":
  [1, 2, 3, 4],
  "multi-line":
  [1, 2,
  3, 4]
}
```

##### `@prepend_input_softline`

```json
{
  "single-line":
  [1 ,2 ,3 ,4],
  "multi-line":
  [1 ,2 ,3
  ,4]
}
```

### Custom scopes and softlines

So far, we've expanded softlines into line breaks depending on whether
the CST node they are associated with is multi-line. Sometimes, CST
nodes define scopes that are either too big or too small for our needs.
For instance, consider this piece of OCaml code:

```ocaml
(1,2,
3)
```

Its CST is the following:

```
{Node parenthesized_expression (0, 0) - (1, 2)} - Named: true
  {Node ( (0, 0) - (0, 1)} - Named: false
  {Node product_expression (0, 1) - (1, 1)} - Named: true
    {Node product_expression (0, 1) - (0, 4)} - Named: true
      {Node number (0, 1) - (0, 2)} - Named: true
      {Node , (0, 2) - (0, 3)} - Named: false
      {Node number (0, 3) - (0, 4)} - Named: true
    {Node , (0, 4) - (0, 5)} - Named: false
    {Node number (1, 0) - (1, 1)} - Named: true
  {Node ) (1, 1) - (1, 2)} - Named: false
```

We would want to add a line break after the first comma, but because the
CST structure is nested, the node containing this comma
(`product_expression (0, 1) - (0, 4)`) is *not* multi-line  Only the
top-level node `product_expression (0, 1) - (1, 1)` is multi-line.

To solve this issue, we introduce user-defined scopes and softlines.

#### `@prepend_begin_scope` / `@append_begin_scope` / `@prepend_end_scope` / `@append_end_scope`

These tags are used to define custom scopes. In conjunction with the `#scope_id!
` predicate, they define scopes that can span multiple CST nodes, or only part
of one. For instance, this scope matches anything between parenthesis in a
`parenthesized_expression`:

```scheme
(parenthesized_expression
  "(" @append_begin_scope
  ")" @prepend_end_scope
  (#scope_id! "tuple")
)
```

#### Scoped softlines

We have four predicates that insert softlines in custom scopes, in
conjunction with the `#scope_id!` predicate:

* `@prepend_empty_scoped_softline`
* `@prepend_spaced_scoped_softline`
* `@append_empty_scoped_softline`
* `@append_spaced_scoped_softline`

When one of these scoped softlines is used, their behaviour depends on
the innermost encompassing scope with the corresponding `scope_id`. If
that scope is multi-line, the softline expands into a line break. In any
other regard, they behave as their non-`scoped` counterparts.

#### Example

This Tree-sitter query:

```scheme
(#language! ocaml)

(parenthesized_expression
  "(" @append_begin_scope @append_empty_softline @append_indent_start
  ")" @append_end_scope @prepend_empty_softline @prepend_indent_end
  (#scope_id! "tuple")
)

(product_expression
  "," @append_spaced_scoped_softline
  (#scope_id! "tuple")
)
```

...formats this piece of code:

```ocaml
(1,2,
3)
```

...as:

```ocaml
(
  1,
  2,
  3
)
```

...while the single-lined `(1, 2, 3)` is kept as is.

If we used `@append_spaced_softline` rather than
`@append_spaced_scoped_softline`, the `1,` would be followed by a space rather
than a newline, because it's inside a single-line `product_expression`.

### Testing context with predicates

Sometimes, similarly to what happens with softlines, we want a query to match
only if the context is single-line, or multi-line. Topiary has several
predicates that achieve this result.

### `#single_line_only!` / `#multi_line_only!`

These predicates allow the query to trigger only if the matched nodes are in a
single-line (resp. multi-line) context.

#### Example

```scheme
; Allow (and enforce) the optional "|" before the first match case
; in OCaml if and only if the context is multi-line
(
  "with"
  .
  "|" @delete
  .
  (match_case)
  (#single_line_only!)
)

(
  "with"
  .
  "|"? @do_nothing
  .
  (match_case) @prepend_delimiter
  (#delimiter! "| ")
  (#multi_line_only!)
)
```

### `#single_line_scope_only!` / `#multi_line_scope_only!`

These predicates allow the query to trigger only if the associated custom scope
containing the matched nodes are is single-line (resp. multi-line).

#### Example

```scheme
; Allow (and enforce) the optional "|" before the first match case
; in function expressions in OCaml if and only if the scope is multi-line
(function_expression
  (match_case)? @do_nothing
  .
  "|" @delete
  .
  (match_case)
  (#single_line_scope_only! "function_definition")
)
(function_expression
  "|"? @do_nothing
  .
  (match_case) @prepend_delimiter
  (#multi_line_scope_only! "function_definition")
  (#delimiter! "| ") ; sic
)
```

### `@prepend_begin_measuring_scope` / `@append_begin_measuring_scope` / `@prepend_end_measuring_scope` / `@append_end_measuring_scope`

Sometimes, custom scopes are not enough: we may want to format a node depending on the multi-line-ness of a piece of code that does not include the node in question. For instance, consider this function application in OCaml:
```ocaml
foo bar (fun x -> qux)
```
We may also want to format it as any of the following two, depending on the actual length of `foo`, `bar`, and `qux`:
```ocaml
foo bar (fun x ->
  qux
)
```
```ocaml
foo
  bar
  (fun x ->
    qux
  )
```
Consider the indentation of `(fun x -> qux)`: if `foo bar` is single-line, we don't want to indent it. But if `foo bar` is multi-line, we do want to indent it.

Because custom scopes can only impact the behaviour of nodes inside the scope, we can't use them to solve this issue. This is why we need `measuring_scope`.

Measuring scopes are opened/closed with a similar syntax as "regular" custom scopes, with any of the following tags, in conjunction with the `#scope_id!` predicate:

* `@prepend_begin_measuring_scope`
* `@append_begin_measuring_scope`
* `@prepend_end_measuring_scope`
* `@prepend_begin_measuring_scope`

Measuring scopes behave as follows:
* A measuring scope must always be contained in a regular custom scope with the same `#scope_id!`. There can't be two measuring scopes with the same `#scope_id!` inside the same regular custom scope.
* If a regular custom scope contains a measuring scope, then all tags contained in the regular scope that depend on its multi-line-ness will instead depend on the multi-line-ness of the measuring scope (hence the name: the inner, measuring scope measures the multi-line-ness of the outer, regular scope).

#### Example

The example below solves the problem of indenting function application in OCaml stated above, using measuring scopes.
```scheme
(application_expression
  .
  (_) @append_indent_start @prepend_begin_scope @prepend_begin_measuring_scope
  (#scope_id! "function_application")
  (_) @append_end_scope
  .
)
; The end of the measuring scope depends on the last argument: if it's a function,
; end it before the function, otherwise end it after the last argument. In that case,
; it's the same as the regular scope.
(application_expression
  (#scope_id! "function_application")
  (_
    [
      (fun_expression)
      (function_expression)
    ]? @do_nothing
  ) @append_end_measuring_scope
  .
)
(application_expression
  (#scope_id! "function_application")
  (_
    [
      (fun_expression)
      (function_expression)
    ] @prepend_end_measuring_scope
  )
  .
)
; If the measuring scope is single-line, end indentation _before_ the last node.
; Otherwise, end the indentation after the last node.
(application_expression
  (#multi_line_scope_only! "function_application")
  (_) @append_indent_end
  .
)
(application_expression
  (#single_line_scope_only! "function_application")
  (_) @prepend_indent_end
  .
)
```

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
