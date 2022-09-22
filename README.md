# Tree-sitter experiment

This project attempts to answer the question if it is possible to create a
uniform formatter for simple languages using the
[Tree-sitter](https://tree-sitter.github.io/tree-sitter/) ecosystem.

## Example

The program can be run like this:

```bash
echo '{"foo":"bar"}' | tree-sitter-formatter --language json
```

Or, if you have Rust or Nix installed, like this:

```bash
echo '{"foo":"bar"}' | cargo run -- --language json
echo '{"foo":"bar"}' | nix run . -- --language json
```

It will output the following formatted code:

```json
{
    "foo": "bar"
}
```

Insert `RUST_LOG=debug` in front of `tree-sitter-formatter` (or `cargo` or
`nix`) if you want to enable debug logging.

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

For each match that we process, we only handle the last capture. That enables us
to do this:

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

If there isn't already a semicolon immediately after the field_declaration,
`@do_nothing` will not be captured, only `@append_delimiter`. Since the last
capture is `@append_delimiter`, that will be processed. If, on the other hand,
there already was a semi-colon, `@do_nothing` would be captured, and we would do
nothing. We may change this so that we can indeed handle several captures per
match. That issue is tracked in
https://github.com/tweag/tree-sitter-formatter/issues/73.

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

### @append_delimiter

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

Note that this would be the same as `@append_space` with space as delimiter.

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

## Contributing

Issues and pull requests are welcome! If you have Nix installed, you can start a
development shell with Rust like this:

```bash
nix develop
```

### Performance

You can check performance before or after changes by running `cargo bench`.

If you do `cargo install flamegraph`, you can generate a performance flamegraph
like this:

```bash
CARGO_PROFILE_RELEASE_DEBUG=true cargo flamegraph -- -l ocaml < tests/samples/input/ocaml.ml > formatted.ml
```
