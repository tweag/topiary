# Topiary

![Latest Release][badge-release]
![CI Status][badge-ci]

* [Topiary web site][topiary-website]
* [Topiary playground][topiary-playground]

Topiary aims to be a uniform formatter for simple languages, as part of
the [Tree-sitter] ecosystem. It is named after the art of clipping or
trimming trees into fantastic shapes.

Topiary is designed for formatter authors and formatter users. Authors
can create a formatter for a language without having to write their own
formatting engine or even their own parser. Users benefit from uniform
code style and, potentially, the convenience of using a single formatter
tool, across multiple languages over their codebases, each with
comparable styles applied.

## Motivation

The style in which code is written has, historically, been mostly left
to personal choice. Of course, this is subjective by definition and has
led to many wasted hours reviewing formatting choices, rather than the
code itself. Prescribed style guides were an early solution to this,
spawning tools that lint a developer's formatting and ultimately leading
to automatic formatters. The latter were popularised by
[`gofmt`][gofmt], whose developers had [the insight][gofmt-slides] that
"good enough" uniform formatting, imposed on a codebase, largely
resolves these problems.

Topiary follows this trend by aspiring to be a "universal formatter
engine", which allows developers to not only automatically format their
codebases with a uniform style, but to define that style for new
languages using a [simple DSL][tree-sitter-query]. This allows for the
fast development of formatters, providing a [Tree-sitter
grammar][tree-sitter-parsers] is defined for that language.

## Design Principles

Topiary has been created with the following goals in mind:

* Use [Tree-sitter] for parsing, to avoid writing yet another grammar
  for a formatter.

* Expect idempotency. That is, formatting of already-formatted code
  doesn't change anything.

* For bundled formatting styles to meet the following constraints:

  * Be compatible with attested formatting styles used for that language
    in the wild.

  * Be faithful to the author's intent: if code has been written such
    that it spans multiple lines, that decision is preserved.

  * Minimise changes between commits such that diffs focus mainly on the
    code that's changed, rather than superficial artefacts. That is, a
    change on one line won't influence others, while the formatting
    won't force you to make later, cosmetic changes when you modify your
    code.

  * Be well-tested and robust, so that the formatter can be trusted in
    large projects.

* For end users -- i.e., not formatting style authors -- the formatter
  should:

  * Prescribe a formatting style that, while customisable, is uniform
    and "good enough" for their codebase.

  * Run efficiently.

  * Afford simple integration with other developer tools, such as
    editors and language servers.

## Language Support
<!-- Update this section as necessary on new developments/releases -->

[For now][topiary-issue4], the Tree-sitter grammars for the languages
that Topiary targets are statically linked. The formatting styles for
these languages come in two levels of maturity: supported and
experimental.

#### Supported

These formatting styles cover their target language and fulfil Topiary's
stated design goals. They are exposed, in Topiary, through a command
line flag.

* [OCaml] (both implementations and interfaces)
* [Nickel]
* [JSON]
* [TOML]

#### Experimental

These languages' formatting styles are subject to change and/or not yet
considered production-ready. They can be accessed in Topiary by
specifying the path to their query files.

* [Rust]
* [Bash]
* [Tree Sitter Queries][tree-sitter-query]

## Getting Started

### Installing

The project can be built and installed with Cargo from the repository
directory:

```bash
cargo install --path topiary-cli
```

Topiary needs to find the language query files (`.scm`) to function properly. By
default, `topiary` looks for a `languages` directory in the current working
directory.

This won't work if you are running Topiary from another directory than this
repository. In order to use Topiary without restriction, **you must set the
environment variable `TOPIARY_LANGUAGE_DIR` to point to the directory where
Topiary's language query files (`.scm`) are located**. By default, you should
set it to `<local path of the topiary repository>/languages`, for example:

```console
export TOPIARY_LANGUAGE_DIR=/home/me/tools/topiary/languages
topiary -i -f ./projects/helloworld/hello.ml
```

`TOPIARY_LANGUAGE_DIR` can alternatively be set at build time. Topiary will pick
the correspond path up and embed it into the `topiary` binary. In that case, you
don't have to worry about making `TOPIARY_LANGUAGE_DIR` available at run-time
anymore. When `TOPIARY_LANGUAGE_DIR` has been set at build time and is set at
run-time as well, the run-time value takes precedence.

See [`CONTRIBUTING.md`][contributing] for details on setting up a
development environment.

### Setting up as pre-commit hook

Topiary integrates seamlessly with [pre-commit-hooks.nix]: add Topiary as input
to your flake and, in [pre-commit-hooks.nix]'s setup, use:

``` nix
pre-commit-check = nix-pre-commit-hooks.run {
  hooks = {
    nixfmt.enable = true; ## keep your normal hooks
    ...
    ## Add the following:
    topiary = topiary.lib.${system}.pre-commit-hook;
  };
};
```

[pre-commit-hooks.nix]: https://github.com/cachix/pre-commit-hooks.nix

### Usage

```
CLI app for Topiary, the universal code formatter.

Usage: topiary [OPTIONS] <--language <LANGUAGE>|--input-file <INPUT_FILE>>

Options:
  -l, --language <LANGUAGE>          Which language to parse and format [possible values: json, nickel, ocaml, ocaml-interface, toml]
  -f, --input-file <INPUT_FILE>      Path to an input file. If omitted, or equal to "-", read from standard input
  -q, --query <QUERY>                Which query file to use
  -o, --output-file <OUTPUT_FILE>    Path to an output file. If omitted, or equal to "-", write to standard output
  -i, --in-place                     Format the input file in place
  -v, --visualise[=<OUTPUT_FORMAT>]  Visualise the syntax tree, rather than format [possible values: json, dot]
  -s, --skip-idempotence             Do not check that formatting twice gives the same output
      --output-configuration         Output the full configuration to stderr before continuing
  -t, --tolerate-parsing-errors      Format as much as possible even if some of the input causes parsing errors
  -h, --help                         Print help
  -V, --version                      Print version
```

Language selection is based on precedence, in the following order:
* A specified language
* Detected from the input file's extension
* A specified query file

#### Exit Codes

The Topiary process will exit with a zero exit code upon successful
formatting. Otherwise, the following exit codes are defined:

| Reason                       | Code |
| :--------------------------- | ---- |
| Unspecified error            |    1 |
| CLI argument parsing error   |    2 |
| I/O error                    |    3 |
| Topiary query error          |    4 |
| Source parsing error         |    5 |
| Language detection error     |    6 |
| Idempotency error            |    7 |
| Unspecified formatting error |    8 |

#### Example

Once built, the program can be run like this:

```bash
echo '{"foo":"bar"}' | topiary --language json
```

`topiary` can also be built and run from source via either Cargo or Nix,
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

## Configuration
Topiary is configured using `languages.toml` files. There are three
locations where Topiary checks for such a file.

### Locations
At buildtime the [languages.toml](./languages.toml) in the root of
this repository is included into Topiary. This file is parsed at
runtime. The purpose of this `languages.toml` file is to provide sane
defaults for users of Topiary (both the library and the binary).

The other two are read by the Topiary binary at runtime and allow the user to
configure Topiary to their needs. The first is intended to be user specific, and
can thus be found in the configuration directory of the OS:
```
Unix: /home/alice/.config/topiary/languages.toml
Windows: C:\Users\Alice\AppData\Roaming\Topiary\config\languages.toml
MacOS: /Users/Alice/Library/Application Support/Topiary/languages.toml
```
This file is not automatically created by Topiary.

The last location is intended to be a project-specific settings file for
Topiary. When running Topiary in some directory, it will look up in the file
tree until it finds a .topiary directory. It will then read the `languages.toml`
file present in that directory.

The Topiary binary parses these file in the following order, any configuration
options defined earlier are overwritten by those defined later.

1. The builtin configuration file
2. The user configuration file in the OS's configuration directory
3. The project specific topiary configuration

### Configuration Options
The configuration file contains a list of languages, each language configuration
headed by ``[[language]]``. For instance, the one for Nickel is defined as such:
```toml
[[language]]
name = "nickel"
extensions = ["ncl"]
```

The `name` field is used by Topiary to associate the language entry with the
query file and tree-sitter grammar. This field should be written lowercase.
The `name` field is mandatory for every ``[[language]]`` block in every
configuration file.

The list of extensions is mandatory for every language, but does not necessarily
need to exist in every configuration file. It is sufficient if, for every
language, there is a single configuration file that defines the list of
extensions for that language.

A final optional field called `indent` exists to define the indentation method
for that language. Topiary defaults to two spaces `"  "` if it cannot find the
indent field in any configuration file for a specific language.

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

Note that a capture is put after the node it is associated with. If you
want to put a space in front of a node, you do it like this:

```scheme
(infix_operator) @prepend_space
```

This, on the other hand, will not work:

```scheme
@append_space (infix_operator)
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

### `@append_multiline_delimiter` / `@prepend_multiline_delimiter`

The matched nodes will have a multi-line-only delimiter appended to
them. It will be printed only in multi-line nodes, and omitted in
single-line nodes. The delimiter must be specified using the predicate
`#delimiter!`.

#### Example

```scheme
; Add a semicolon at the end of lists only if they are multi-line, to avoid [1; 2; 3;].
(list_expression
  (#delimiter! ";")
  (_) @append_multiline_delimiter
  .
  ";"? @do_nothing
  .
  "]"
  .
)
```

If there is already a semicolon, the `@do_nothing` instruction will be
activated and prevent the other instructions in the query (the
`@append_multiline_delimiter`, here) from applying. Likewise, if the
node is single-line, the delimiter will not be appended either.

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

### `@single_line_no_indent`

The matched node will be printed alone, on a single line, with no indentation.

#### Example

```scheme
(#language! ocaml)
; line number directives must be alone on their line, and can't be indented
(line_number_directive) @single_line_no_indent
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

#### `@begin_scope` / `@end_scope`

`@begin_scope` and `@end_scope` tags are used to define custom scopes.
In conjunction with the `#scope_id!` predicate, they define scopes that
can span multiple CST nodes, or only part of one. For instance, this
scope matches anything between parenthesis in a
`parenthesized_expression`:

```scheme
(parenthesized_expression
  "(" @begin_scope
  ")" @end_scope
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
  "(" @begin_scope @append_empty_softline @append_indent_start
  ")" @end_scope @prepend_empty_softline @prepend_indent_end
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

### Testing context with predicates

Sometimes, similarly to what happens with softlines, we want a query to match only if the context is single-line, or multi-line. Topiary has several predicates that achieve this result.

### `#single_line_only!` / `#multi_line_only!`

These predicates allow the query to trigger only if the matched nodes are in a single-line (resp. multi-line) context.

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

These predicates allow the query to trigger only if the associated custom scope containing the matched nodes are is single-line (resp. multi-line).

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

## Suggested workflow

In order to work productively on query files, the following is one
suggested way to work:

1. Add a sample file to `topiary/tests/samples/input`.

2. Copy the same file to `topiary/tests/samples/expected`, and make any changes
   to how you want the output to be formatted.

3. If this is a new language, add its Tree-sitter grammar, extend
   `crate::language::Language` and process it everywhere, then make a
   mostly empty query file with just the `(#language!)` configuration.

4. Run `RUST_LOG=debug cargo test`.

   Provided it works, it should output a lot of log messages. Copy that
   output to a text editor. You are particularly interested in the CST
   output that starts with a line like this: `CST node: {Node
   compilation_unit (0, 0) - (5942, 0)} - Named: true`.

   :bulb: As an alternative to using the debugging output, the
   `--visualise` command line option exists to output the Tree-sitter
   syntax tree in a variety of formats.

5. The test run will output all the differences between the actual
   output and the expected output, e.g. missing spaces between tokens.
   Pick a difference you would like to fix, and find the line number and
   column in the input file.

   :bulb: Keep in mind that the CST output uses 0-based line and column
   numbers, so if your editor reports line 40, column 37, you probably
   want line 39, column 36.

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

Nix users may also find the `playground.sh` script to be helpful in
aiding the interactive development of query files. When run in a
terminal, it will format the given source input with the requested query
file, updating the output on any inotify event against those files.

```
Usage: playground.sh (LANGUAGE | QUERY_FILE) [INPUT_SOURCE]

LANGUAGE can be one of the supported languages (e.g., "ocaml", "rust",
etc.); alternatively, give the path to the query file itself, as
QUERY_FILE.

The INPUT_SOURCE is optional. If not specified, it defaults to trying
to find the bundled integration test input file for the given language.
```

For example, the playground can be run in a tmux pane, with your editor
of choice open in another.

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

[badge-ci]: https://github.com/tweag/topiary/actions/workflows/ci.yml/badge.svg
[badge-release]: https://img.shields.io/github/v/release/tweag/topiary?display_name=release&sort=semver
[bash]: https://www.gnu.org/software/bash
[contributing]: CONTRIBUTING.md
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
[ocp-indent]: https://www.typerex.org/ocp-indent.html
[ormolu]: https://github.com/tweag/ormolu
[prettier]: https://prettier.io/
[rust]: https://www.rust-lang.org
[rustfmt]: https://rust-lang.github.io/rustfmt
[shfmt]: https://github.com/mvdan/sh
[toml]: https://toml.io
[topiary-issue4]: https://github.com/tweag/topiary/issues/4
[topiary-playground]: https://topiary.tweag.io/playground
[topiary-website]: https://topiary.tweag.io
[tree-sitter-parsers]: https://tree-sitter.github.io/tree-sitter/#available-parsers
[tree-sitter-playground]: https://tree-sitter.github.io/tree-sitter/playground
[tree-sitter-query]: https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries
[tree-sitter]: https://tree-sitter.github.io/tree-sitter
[treefmt]: https://github.com/numtide/treefmt
