# Topiary

[![Latest Release][badge-release]][badge-release-link]
[![CI Status][badge-ci]][badge-ci-link]
[![Discord][badge-discord]][badge-discord-link]

* [Topiary web site][topiary-website]
* [Topiary playground][topiary-playground]

## Language Support
<!-- Update this section as necessary on new developments/releases -->

The formatting styles for these languages come in two levels of maturity:
supported and experimental.

#### Supported

These formatting styles cover their target language and fulfil Topiary's
stated design goals. They are exposed, in Topiary, through the
`--language` command line flag, or language detection (based on file
extension).

* [Bash]
* [JSON]
* [Nickel]
* [OCaml] (both implementations and interfaces)
* [OCamllex]
* [TOML]
* [Tree Sitter Queries][tree-sitter-query]

#### Contributed

These languages' formatting styles have been generously provided by
external contributors. They are built in, by default, so are exposed in
the same way as supported languages.

* [CSS] by @lavigneer
* [OpenSCAD] by @mkatychev
* [SDML] by @johnstonskj

#### Experimental

These languages' formatting styles are subject to change and/or not yet
considered production-ready. They are _not_ built by default and are
gated behind a feature flag (either `experimental`, for all of them, or
by their individual name). Once included, they can be accessed in
Topiary in the usual way.

* [Rust]

## Design

As long as there is a [Tree-sitter grammar][tree-sitter-parsers] defined
for a language, Tree-sitter can parse it and provide a concrete syntax
tree (CST). Tree-sitter will also allow us to run queries against this
tree. We can make use of that to define how a language should be
formatted. Here's an example query:

```scheme
[
  (infix_operator)
  "if"
  ":"
] @append_space
```

This will match any node that the grammar has identified to be an
`infix_operator`, as well as any anonymous node containing `if` or `:`.
The match will be captured with the name `@append_space`. Our formatter
runs through all matches and captures, and when we process any capture
called `@append_space`, we will append a space after the matched node.

The formatter goes through the CST nodes and detects all that are
spanning more than one line. This is interpreted to be an indication
from the programmer who wrote the input that the node in question should
be formatted as multi-line. Any other nodes will be formatted as
single-line. Whenever a query match has inserted a _softline_, it will
be expanded to a newline if the node is multi-line, or to a space or
nothing if the node is single-line, depending on whether
`@append_spaced_softline` or `@append_empty_softline` was used.

Before rendering the output, the formatter will do a number of cleanup
operations, such as reducing consecutive spaces and newlines to one,
trimming spaces at end of lines and leading and trailing blanks lines,
and ordering indenting and newline instructions consistently.

This means that you can for example prepend and append spaces to `if`
and `true`, and we will still output `if true` with just one space
between the words.

## Supported capture instructions

This assumes you are already familiar with the [Tree-sitter query
language][tree-sitter-query].

### A note on anchors
The behaviour of "anchors" can be counterintuitive. Consider, for instance, the
following query:
```scheme
(
  (list_entry) @append_space
  .
)
```
One might assume that this query only matches the final element in the list but
this is not true. Since we did not explicitly march a parent node, the engine
will match on every `list_entry`. After all, the when looking only at the nodes
in the query, the `list_entry` is indeed the last node.

To resolve this issue, match explicitly on the parent node:
```scheme
(list
  (list_entry) @append_space
  .
)
```

Or even implicitly:
```scheme
(_
  (list_entry) @append_space
  .
)
```

Note that a capture is put after the node it is associated with. If you
want to put a space in front of a node, you do it like this:

```scheme
(infix_operator) @prepend_space
```

This, on the other hand, will not work:

```scheme
@append_space (infix_operator)
```

### `@leaf`

Some nodes should not have their contents formatted at all; the classic
example being string literals. The `@leaf` capture will mark such nodes
as leaves -- even if they admit their own structure -- and leave them
unformatted.

#### Example

```scheme
; Don't format strings or comments
[
  (string)
  (comment)
] @leaf
```

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

### `@append_space` / `@prepend_space`

The matched nodes will have a space appended or prepended to them. Note
that this is the same as `@append_delimiter` / `@prepend_delimiter`,
with space as delimiter.

#### Example

```scheme
[
  (infix_operator)
  "if"
  ":"
] @append_space
```

### `@append_antispace` / `@prepend_antispace`

It is often the case that tokens need to be juxtaposed with spaces,
except in a few isolated contexts. Rather than writing complicated rules
that enumerate every exception, an "antispace" can be inserted with
`@append_antispace` / `@prepend_antispace`; this will destroy any spaces
(not newlines) from that node, including those added by other formatting
rules.

#### Example

```scheme
[
  ","
  ";"
  ":"
  "."
] @prepend_antispace
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

### `@do_nothing`

If any of the captures in a query match are `@do_nothing`, then the
match will be ignored.

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

### `@lower_case`/`@upper_case`
Set the capitalization of all text in the matched node and its children.
Use this with care in languages that are case sensitive.

```scheme
; example for SQL, since that's where this makes sense.
; I am using the grammar linked below
; https://github.com/DerekStride/tree-sitter-sql/tree/main

; make keywords select,from lowercase.
[
 (keyword_select)
 (keyword_from)
] @lower_case
; make keyword WHERE uppercase
(keyword_where) @upper_case
```

### `#query_name!`

When the logging verbosity is set to `-vv` or higher, Topiary outputs information about which queries are matched, for instance:
```
[2024-10-08T15:48:13Z INFO  topiary_core::tree_sitter] Processing match: LocalQueryMatch { pattern_index: 17, captures: [ {Node "," (1,3) - (1,4)} ] } at location (286,1)
```
The predicate `#query_name!` takes a string argument, is optional, and can be added to any query.
It will modify the log line to display its argument.

#### Example

Considering the log line above, and let us assume that the query at `location (286,1)` is:

```scheme
(
  "," @append_space
  .
  (_)
)
```
If we add a `query_name` predicate:
```scheme
(
  (#query_name! "comma spacing")
  "," @append_space
  .
  (_)
)
```
Then the log line will become:
```
[2024-10-08T15:48:13Z INFO  topiary_core::tree_sitter] Processing match of query "comma spacing": LocalQueryMatch { pattern_index: 17, captures: [ {Node "," (1,3) - (1,4)} ] } at location (286,1)
```

### Query and capture precedence

Formatting is not necessarily invariant over the order of queries. For
example, queries that add delimiters or remove nodes can have a
different effect on the formatted output depending on the order in which
they appear in the query file.

Consider, say, the following two queries for the Bash grammar:

```scheme
; Query A: Append semicolon
(
  (word) @append_delimiter
  .
  ";"? @do_nothing

  (#delimiter! ";")
)

; Query B: Surround with quotes
(
  "\""? @do_nothing
  .
  (word) @prepend_delimiter @append_delimiter
  .
  "\""? @do_nothing

  (#delimiter! "\"")
)
```

In the order presented above (`A`, then `B`), then the input `foo` will
be formatted as:

```
"foo;"
```

In the opposite order (`B`, then `A`), Topiary will however produce the
following output:

```
"foo";
```

A similar consideration exists for captures. That is, while most
captures do not meaningfully affect one another, there are three notable
exceptions:

1. `@do_nothing` will cancel all other captures in a matched query. This
   takes the highest priority.

2. `@delete` will delete any matched node, providing the matching query
   is not cancelled.

3. `@leaf` will suppress formatting within that node, even if it admits
   some internal structure. However, leaf nodes are still subject to
   deletion.

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

### Syntax Tree Visualisation

To support the development of formatting queries, the Tree-sitter syntax
tree for a given input can be produced using the `--visualise` CLI
option.

This currently supports JSON output, covering the same information as
the debugging output, as well as GraphViz DOT output, which is useful
for generating syntax diagrams. (Note that the text position
serialisation in the visualisation output is 1-based, unlike the
debugging output's 0-based position.)

### Terminal-Based Playground

Nix users may also find the `bin/playground.sh` script to be helpful in
aiding the interactive development of query files. When run in a
terminal, inside the Nix development shell, it will format the given
source input with the requested query file, updating the output on any
inotify event against those files.

```
Usage: playground LANGUAGE [QUERY_FILE] [INPUT_SOURCE]

LANGUAGE can be one of the supported languages (e.g., "ocaml", "rust",
etc.). The packaged formatting queries for this language can be
overridden by specifying a QUERY_FILE.

The INPUT_SOURCE is optional. If not specified, it defaults to trying
to find the bundled integration test input file for the given language.
```

For example, the playground can be run in a tmux pane, with your editor
of choice open in another.

> [!WARNING]
> The use of inotify limits this tool to Linux systems, only.

## Related Tools

### Tree-Sitter Specific

* [Syntax Tree Playground][tree-sitter-playground]: An interactive,
  online playground for experimenting with Tree-sitter and its query
  language.
* [Neovim Treesitter Playground][nvim-treesitter]: A Tree-sitter
  playground plugin for Neovim.
* [Difftastic]: A tool that utilises Tree-sitter to perform syntactic
  diffing.

### Meta and Multi-Language Formatters

* [format-all]: A formatter orchestrator for Emacs.
* [null-ls.nvim]: An LSP framework for Neovim that facilitates formatter
  orchestration.
* [prettier]: A formatter with support for multiple (web-development
  related) languages.
* [treefmt]: A general formatter orchestrator, which unifies formatters
  under a common interface.

### Related Formatters

* [gofmt]: The de facto standard formatter for Go, and major source of
  inspiration for the style of our formatters.
* [ocamlformat]: A formatter for OCaml.
* [ocp-indent]: A tool to indent OCaml code.
* [Ormolu]: Our formatter for Haskell, which follows similar design
  principles as Topiary.
* [rustfmt]: The de facto standard formatter for Rust.
* [shfmt]: A parser, formatter and interpreter for Bash et al.

<!-- Links -->

[badge-ci]: https://img.shields.io/github/actions/workflow/status/tweag/topiary/ci.yml?logo=github
[badge-ci-link]: https://github.com/tweag/topiary/actions/workflows/ci.yml
[badge-discord]: https://img.shields.io/discord/1174731094726295632?logo=discord
[badge-discord-link]: https://discord.gg/FSnkvNyyzC
[badge-release]: https://img.shields.io/github/v/release/tweag/topiary?display_name=release&logo=github
[badge-release-link]: https://github.com/tweag/topiary/releases/latest
[bash]: https://www.gnu.org/software/bash
[contributing]: CONTRIBUTING.md
[css]: https://en.wikipedia.org/wiki/CSS
[difftastic]: https://difftastic.wilfred.me.uk
[format-all]: https://melpa.org/#/format-all
[gofmt-slides]: https://go.dev/talks/2015/gofmt-en.slide#1
[gofmt]: https://pkg.go.dev/cmd/gofmt
[json]: https://www.json.org
[nickel]: https://nickel-lang.org
[null-ls.nvim]: https://github.com/jose-elias-alvarez/null-ls.nvim
[nvim-treesitter]: https://github.com/nvim-treesitter/playground
[ocaml]: https://ocaml.org
[ocamlformat]: https://github.com/ocaml-ppx/ocamlformat
[ocamllex]: https://v2.ocaml.org/manual/lexyacc.html
[ocp-indent]: https://www.typerex.org/ocp-indent.html
[openscad]: https://en.wikipedia.org/wiki/OpenSCAD
[ormolu]: https://github.com/tweag/ormolu
[prettier]: https://prettier.io/
[rust]: https://www.rust-lang.org
[rustfmt]: https://rust-lang.github.io/rustfmt
[sdml]: https://sdml.io/
[shfmt]: https://github.com/mvdan/sh
[toml]: https://toml.io
[topiary-playground]: https://topiary.tweag.io/playground
[topiary-website]: https://topiary.tweag.io
[tree-sitter-parsers]: https://tree-sitter.github.io/tree-sitter/#available-parsers
[tree-sitter-playground]: https://tree-sitter.github.io/tree-sitter/playground
[tree-sitter-query]: https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries
[tree-sitter]: https://tree-sitter.github.io/tree-sitter
[treefmt]: https://github.com/numtide/treefmt
