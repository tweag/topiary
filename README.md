# Topiary

![CI Status](https://github.com/tweag/topiary/actions/workflows/ci.yml/badge.svg)

Topiary aims to be a uniform formatter for simple languages, as part of
the [Tree-sitter](https://tree-sitter.github.io/tree-sitter/) ecosystem.
It is named after the art of clipping or trimming trees into fantastic
shapes.

## Getting Started

### Installing

The project can be built and installed with Cargo from the repository
directory:

```bash
cargo install --path .
```

See the [Contributing](#contributing) section for details on setting up
a development environment.

### Usage

    topiary [OPTIONS] <--language <LANGUAGE>
                      |--query <QUERY>
                      |--input-file <INPUT_FILE>>

Options:

* `-h`, `--help`\
  Print help information

* `-i`, `--input-file <INPUT_FILE>`\
  Path to an input file. If omitted, or equal to "-", read from standard
  input

* `-l`, `--language <LANGUAGE>`\
  Which language to parse and format [possible values: json, toml]

* `-o`, `--output-file <OUTPUT_FILE>`\
  Path to an output file. If omitted, or equal to "-", write to standard
  output

* `-q`, `--query <QUERY>`\
  Which query file to use

* `-s`, `--skip-idempotence`\
  Do not check that formatting twice gives the same output

* `-V`, `--version`\
  Print version information

Language selection is based on precedence, in the following order:
* A specified query file
* A specified language
* Detected from the input file's extension

#### Example

Once built, the program can be run like this:

```bash
echo '{"foo":"bar"}' | topiary --language json
```

`topiary` can also be built and run from source via either Rust or Nix,
if you have those installed:

```bash
echo '{"foo":"bar"}' | cargo run -- --language json
echo '{"foo":"bar"}' | nix run . -- --language json
```

It will output the following formatted code:

```json
{ "foo": "bar" }
```

Set the `RUST_LOG=debug` environment variable if you want to enable
debug logging.

## Design

As long as there is a [Tree-sitter
grammar](https://tree-sitter.github.io/tree-sitter/#available-parsers) defined
for a language, Tree-sitter can parse it and provide a concrete syntax tree
(CST). Tree-sitter will also allow us to run queries against this tree. We can
make use of that to define how a language should be formatted. Here's an example
query:

```scheme
[
  (infix_operator)
  "if"
  ":"
] @append_space
```

This will match any node that the grammar has identified to be an
`infix_operator`, as well as any anonymous node containing `if` or `:`. The
match will be captured with the name `@append_space`. Our formatter runs through
all matches and captures, and when we process any capture called
`@append_space`, we will append a space after the matched node.

The formatter goes through the CST nodes and detects all that are spanning more
than one line. This is interpreted to be an indication from the programmer who
wrote the input that the node in question should be formatted as multi-line. Any
other nodes will be formatted as single-line. Whenever a query match has
inserted a _softline_, it will be expanded to a newline if the node is
multi-line, or to a space or nothing if the node is single-line, depending on
whether `@append_spaced_softline` or `@append_empty_softline` was used.

Before rendering the output, the formatter will do a number of cleanup
operations, such as reducing consecutive spaces and newlines to one, trimming
spaces at end of lines, and ordering indenting and newline instructions
consistently.

This means that you can for example prepend and append spaces to `if` and
`true`, and we will still output `if true` with just one space between the
words.

## Supported capture instructions

This assumes you are already familiar with the [Tree-sitter query
language](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries).

Note that a capture is put after the node it is associated with. If you want to
put a space in front of a node, you do it like this:

```scheme
(infix_operator) @prepend_space
```

This, on the other hand, will not work:

```scheme
@append_space (infix_operator)
```

### Configuration

At the top of a query file you can set some configuration options like this:

```scheme
; Configuration
(#language! rust)
(#indent-level! 4)
```

The `#language!` predicate must be included in any query file and dictates which
language to parse. The queries themselves will refer to node types that are
specific to this language.

The `#indent-level!` predicate is optional and controls how many spaces to
indent a block whenever `@append_indent_start` or `@prepend_indent_start` is
processed.

### @allow_blank_line_before

The matched nodes will be allowed to have a blank line before them, if specified
in the input. For any other nodes, blank lines will be removed.

#### Example

```scheme
; Allow comments and type definitions to have a blank line above them
[
  (comment)
  (type_definition)
] @allow_blank_line_before
```

### @append_delimiter / @prepend_delimiter

The matched nodes will have a delimiter appended to them. The delimiter must
be specified using the predicate `#delimiter!`.

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

If there is already a semicolon, the `@do_nothing` instruction will be activated
and prevent the other instruction in the query (the `@append_delimiter`) to activate.
Otherwise, the `";"*` captures nothing and in this case the associated instruction (`@do_nothing`) does not activate.

Note that `@append_delimiter` is the same as `@append_space` when the delimiter is set to `" "` (space).

### @append_empty_softline / @prepend_empty_softline

The matched nodes will have an empty softline appended or prepended to them.
This will be expanded to a newline for multi-line nodes and to nothing for
single-line nodes.

#### Example

```scheme
; Put an empty softline before dots, so that in multi-line constructs we start
; new lines for each dot.
(_
  "." @prepend_empty_softline
)
```

### @append_hardline

The matched nodes will have a newline appended to them.

#### Example

```scheme
; Consecutive definitions must be separated by line breaks
(
  (value_definition) @append_hardline
  .
  (value_definition)
)
```

### @append_indent_start / @prepend_indent_start

The matched nodes will trigger indentation before or after them. This will only apply
for lines following, until an indentation end is signalled. If indentation is
started and ended on the same line, nothing will happen. This is useful, because
we get the correct behaviour whether a node is formatted as single-line or
multi-line. It is important that all indentation start and end is balanced.

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

### @append_indent_end / @prepend_indent_end

The matched nodes will trigger that indentation ends before or after them.

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

### @append_input_softline / @prepend_input_softline

The matched nodes will have an input softline appended or prepended to them. An
input softline is a newline if the node has a newline in front of it in the
input document, otherwise it is a space.

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

### @append_space / @prepend_space

The matched nodes will be have a space appended or prepended to them. Note that
this is the same as `@append_delimiter` / `@prepend_delimiter` with space as
delimiter.

#### Example

```scheme
[
  (infix_operator)
  "if"
  ":"
] @append_space
```

### @append_spaced_softline / @prepend_spaced_softline

The matched nodes will have a spaced softline appended or prepended to them.
This will be expanded to a newline for multi-line nodes and to a space for
single-line nodes.

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

### @do_nothing

If any of the captures in a query match are `@do_nothing`, then the match will
be ignored.

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

### Understanding the different newline captures

| Type            | Append/Prepend | Single-Line Context | Multi-Line Context |
| :-------------- | :------------- | :------------------ | :----------------- |
| Hardline        | Append Only    | Newline             | Newline            |
| Empty Softline  | Both           | Nothing             | Newline            |
| Spaced Softline | Both           | Space               | Newline            |
| Input Softline  | Both           | Input-Dependent     | Input-Dependent    |

"Input softlines" are rendered as newlines whenever they proceed a
newline in the input. Otherwise, they are rendered as spaces.

#### Example

Consider the following pseudocode:

```bash
# This is a comment

# Here's another comment
some_syntax # Yet another comment
```

We shall apply the different newline captures to syntactic items and
comments, respectively, to observe their effect. That is, for each
`@CAPTURE` name, we apply the following queries:

```scheme
(syntax_node) @CAPTURE
(comment) @CAPTURE
```

(Note that trailing newlines have been replaced with `␊` so they are not
stripped by GitHub's markdown rendering.)

##### `@append_hardline`

```bash
# This is a comment
# Here's another comment
some_syntax
# Yet another comment
␊
```

##### `@append_empty_softline`

```bash
# This is a comment
# Here's another comment
some_syntax
# Yet another comment
␊
```

##### `@prepend_empty_softline`

```bash
# This is a comment
# Here's another comment
some_syntax
# Yet another comment
```

##### `@append_spaced_softline`

```bash
# This is a comment
# Here's another comment
some_syntax
# Yet another comment
␊
```

##### `@prepend_spaced_softline`

```bash
# This is a comment
# Here's another comment
some_syntax
# Yet another comment
```

##### `@append_input_softline`

```bash
# This is a comment
# Here's another comment
some_syntax # Yet another comment
```

##### `@prepend_input_softline`

```bash
# This is a comment
# Here's another comment
some_syntax # Yet another comment
```

## Suggested workflow

In order to work productively on query files, the following is one suggested way to work:

1. Add a sample file to `tests/samples/input`.
2. Copy the same file to `tests/samples/expected`, and make any changes to how you want the output to be formatted.
3. If this is a new language, add a Tree-sitter grammar, extend `crate::language::Language` and process it everywhere, then make a mostly empty query file with just the `(#language!)` configuration.
4. Run `RUST_LOG=debug cargo test`.
5. Provided it works, it should output a lot of log messages. Copy that output to a text editor. You are particularly interested in the CST output that starts with a line like this: `CST node: {Node compilation_unit (0, 0) - (5942, 0)} - Named: true`.
6. The test run will output all the differences between the actual output and the expected output, e.g. missing spaces between tokens. Pick a difference you would like to fix, and find the line number and column in the input file.
7. Keep in mind that the CST output uses 0-based line and column numbers, so if your editor reports line 40, column 37, you probably want line 39, column 36.
8. In the CST debug output, find the nodes in this region, such as the following:

```
[DEBUG atom_collection] CST node:   {Node constructed_type (39, 15) - (39, 42)} - Named: true
[DEBUG atom_collection] CST node:     {Node type_constructor_path (39, 15) - (39, 35)} - Named: true
[DEBUG atom_collection] CST node:       {Node type_constructor (39, 15) - (39, 35)} - Named: true
[DEBUG atom_collection] CST node:     {Node type_constructor_path (39, 36) - (39, 42)} - Named: true
[DEBUG atom_collection] CST node:       {Node type_constructor (39, 36) - (39, 42)} - Named: true
```

9. This may indicate that you would like spaces after all `type_constructor_path`:

```scheme
(type_constructor_path) @append_space
```

10. Or, more likely, you just want spaces between pairs of them:

```scheme
(
  (type_constructor_path) @append_space
  .
  (type_constructor_path)
)
```

11. Or maybe you want spaces between all children of `constructed_type`:

```scheme
(constructed_type
  (_) @append_space
  .
  (_)
)
```

12. Run `cargo test` again, see if the output is better now, and then go back to step 6.
