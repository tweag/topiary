# Topiary

[![Latest Release][badge-release]][badge-release-link]
[![CI Status][badge-ci]][badge-ci-link]
[![Discord][badge-discord]][badge-discord-link]

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
* [CFML] (ColdFusion Markup Language) by @ghedwards

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
set it to `<local path of the topiary repository>/topiary-queries/queries`, for example:

```sh
export TOPIARY_LANGUAGE_DIR=/home/me/tools/topiary/topiary-queries/queries
topiary fmt ./projects/helloworld/hello.ml
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

The Topiary CLI uses a number of subcommands to delineate functionality.
These can be listed with `topiary --help`; each subcommand then has its
own, dedicated help text.

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:ROOT -->
```
CLI app for Topiary, the universal code formatter.

Usage: topiary [OPTIONS] <COMMAND>

Commands:
  format      Format inputs
  visualise   Visualise the input's Tree-sitter parse tree
  config      Print the current configuration
  prefetch    Prefetch all languages in the configuration
  coverage    Checks how much of the tree-sitter query is used
  completion  Generate shell completion script
  help        Print this message or the help of the given subcommand(s)

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -M, --merge-configuration            Enable merging for configuration files
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
  -V, --version                        Print version
```
<!-- usage:end:ROOT -->

#### Format

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:format -->
```
Format inputs

Usage: topiary format [OPTIONS] <--language <LANGUAGE>|FILES>

Arguments:
  [FILES]...
          Input files and directories (omit to read from stdin)

          Language detection and query selection is automatic, mapped from file extensions defined
          in the Topiary configuration.

Options:
  -t, --tolerate-parsing-errors
          Consume as much as possible in the presence of parsing errors

  -s, --skip-idempotence
          Do not check that formatting twice gives the same output

  -l, --language <LANGUAGE>
          Topiary language identifier (when formatting stdin)

  -q, --query <QUERY>
          Topiary query file override (when formatting stdin)

  -L, --follow-symlinks
          Follow symlinks (when formatting files)

  -C, --configuration <CONFIGURATION>
          Configuration file

          [env: TOPIARY_CONFIG_FILE]

  -M, --merge-configuration
          Enable merging for configuration files

  -v, --verbose...
          Logging verbosity (increased per occurrence)

  -h, --help
          Print help (see a summary with '-h')
```
<!-- usage:end:format -->

When formatting inputs from disk, language selection is detected from
the input files' extensions. To format standard input, you must specify
the `--language` and, optionally, `--query` arguments, omitting any
input files.

> [!TIP]
> `fmt` is a recognised alias of the `format` subcommand.

> [!TIP]
> Topiary will not accept a process substitution (or any other named
> pipe) as formatting input. Instead, arrange for a redirection into
> Topiary's standard input:
>
> ```bash
> # This won't work
> topiary format <(some_command)
>
> # Do this instead
> some_command | topiary format --language LANGUAGE
> ```

> [!NOTE]
> Topiary will skip over some input files under certain conditions,
> which are logged at varying levels:
>
> | Condition                                     | Level   |
> | :-------------------------------------------- | :------ |
> | Cannot access file                            | Error   |
> | Not a regular file (e.g., FIFO, socket, etc.) | Warning |
> | A symlink without `--follow-symlinks`         | Warning |
> | File with multiple (hard) links               | Error   |
> | File does not exist (e.g., broken symlink)    | Error   |

#### Visualise

`topiary visualise` converts the input's Tree-sitter parse tree to a graph
representation in the selected format. By default, Topiary outputs a DOT file,
which can be rendered using a visualisation tool such as the Graphviz suite. For
example, using Graphviz's `dot`: `topiary visualise input.ocaml | dot -T png
-o output.png`.

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:visualise -->
```
Visualise the input's Tree-sitter parse tree

Visualise generates a graph representation of the parse tree that can be rendered by external
visualisation tools, such as Graphviz. By default, the output is in the DOT format.

Usage: topiary visualise [OPTIONS] <--language <LANGUAGE>|FILE>

Arguments:
  [FILE]
          Input file (omit to read from stdin)

          Language detection and query selection is automatic, mapped from file extensions defined
          in the Topiary configuration.

Options:
  -f, --format <FORMAT>
          Visualisation format

          [default: dot]

          Possible values:
          - dot:  GraphViz DOT serialisation
          - json: JSON serialisation

  -l, --language <LANGUAGE>
          Topiary language identifier (when formatting stdin)

  -q, --query <QUERY>
          Topiary query file override (when formatting stdin)

  -C, --configuration <CONFIGURATION>
          Configuration file

          [env: TOPIARY_CONFIG_FILE]

  -M, --merge-configuration
          Enable merging for configuration files

  -v, --verbose...
          Logging verbosity (increased per occurrence)

  -h, --help
          Print help (see a summary with '-h')
```
<!-- usage:end:visualise -->

When visualising inputs from disk, language selection is detected from
the input file's extension. To visualise standard input, you must
specify the `--language` and, optionally, `--query` arguments, omitting
the input file. The visualisation output is written to standard out.

> [!TIP]
> `vis`, `visualize` and `view` are recognised aliases of the
> `visualise` subcommand.

#### Configuration

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:config -->
```
Print the current configuration

Usage: topiary config [OPTIONS]

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -M, --merge-configuration            Enable merging for configuration files
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
```
<!-- usage:end:config -->

> [!TIP]
> `cfg` is a recognised alias of the `config` subcommand.

#### Shell Completion

Shell completion scripts for Topiary can be generated with the
`completion` subcommand. The output of which can be sourced into your
shell session or profile, as required.

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:completion -->
```
Generate shell completion script

Usage: topiary completion [OPTIONS] [SHELL]

Arguments:
  [SHELL]  Shell (omit to detect from the environment) [possible values: bash, elvish, fish,
           powershell, zsh]

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -M, --merge-configuration            Enable merging for configuration files
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
```
<!-- usage:end:completion -->

For example, in Bash:

```bash
source <(topiary completion)
```

#### Prefetching

Topiary dynamically downloads, builds, and loads the tree-sitter grammars. In
order to ensure offline availability or speed up startup time, the grammars can
be prefetched and compiled.

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:prefetch-->
```
Prefetch all languages in the configuration

Usage: topiary prefetch [OPTIONS]

Options:
  -f, --force                          Re-fetch existing grammars if they already exist
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -M, --merge-configuration            Enable merging for configuration files
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
```
<!-- usage:end:prefetch -->

#### Coverage

This subcommand checks how much of the language query file is used to process the input.
Specifically, it checks the percentage of queries in the query file that match the given input,
and prints the queries that don't match anything.

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:coverage-->
```
Checks how much of the tree-sitter query is used

Usage: topiary coverage [OPTIONS] <--language <LANGUAGE>|FILE>

Arguments:
  [FILE]
          Input file (omit to read from stdin)

          Language detection and query selection is automatic, mapped from file extensions defined
          in the Topiary configuration.

Options:
  -l, --language <LANGUAGE>
          Topiary language identifier (when formatting stdin)

  -q, --query <QUERY>
          Topiary query file override (when formatting stdin)

  -C, --configuration <CONFIGURATION>
          Configuration file

          [env: TOPIARY_CONFIG_FILE]

  -M, --merge-configuration
          Enable merging for configuration files

  -v, --verbose...
          Logging verbosity (increased per occurrence)

  -h, --help
          Print help (see a summary with '-h')
```
<!-- usage:end:coverage -->

The `coverage` subcommand will exit with error code `1` if the coverage is less than 100%.

#### Logging

By default, the Topiary CLI will only output error messages. You can
increase the logging verbosity with a respective number of
`-v`/`--verbose` flags:

| Verbosity Flag | Logging Level           |
| :------------- | :---------------------- |
| None           | Errors                  |
| `-v`           | ...and warnings         |
| `-vv`          | ...and information      |
| `-vvv`         | ...and debugging output |
| `-vvvv`        | ...and tracing output   |

#### Exit Codes

The Topiary process will exit with a zero exit code upon successful
formatting. Otherwise, the following exit codes are defined:

| Reason                       | Code |
| :--------------------------- | ---: |
| Negative result              |    1 |
| CLI argument parsing error   |    2 |
| I/O error                    |    3 |
| Topiary query error          |    4 |
| Source parsing error         |    5 |
| Language detection error     |    6 |
| Idempotency error            |    7 |
| Unspecified formatting error |    8 |
| Multiple errors              |    9 |
| Unspecified error            |   10 |

Negative results with error code `1` only happen when Topiary is called
with the `coverage` sub-command, if the input does not cover 100% of the query.

When given multiple inputs, Topiary will do its best to process them
all, even in the presence of errors. Should _any_ errors occur, Topiary
will return a non-zero exit code. For more details on the nature of
these errors, run Topiary at the `warn` logging level (with `-v`).

#### Example

Once built, the program can be run like this:

```bash
echo '{"foo":"bar"}' | topiary fmt --language json
```

`topiary` can also be built and run from source via either Cargo or Nix,
if you have those installed:

```bash
echo '{"foo":"bar"}' | cargo run -- fmt --language json
echo '{"foo":"bar"}' | nix run . -- fmt --language json
```

It will output the following formatted code:

```json
{ "foo": "bar" }
```

## Configuration

Topiary is configured using `languages.ncl` files. The `.ncl` extension relates
to [Nickel](https://nickel-lang.org/), a configuration language created by
Tweag. There are up to four sources where Topiary checks for such a file.

### Configuration Sources

At build time the [languages.ncl](./languages.ncl) in the root of
this repository is embedded into Topiary. This file is parsed at
runtime. The purpose of this `languages.ncl` file is to provide sane
defaults for users of Topiary (both the library and the binary).

The next two are read by the Topiary binary at runtime and allow the user to
configure Topiary to their needs. The first is intended to be user specific, and
can thus be found in the configuration directory of the OS:

| OS      | Typical Configuration Path                                        |
| :------ | :---------------------------------------------------------------- |
| Unix    | `/home/alice/.config/topiary/languages.ncl`                      |
| Windows | `C:\Users\Alice\AppData\Roaming\Topiary\config\languages.ncl`    |
| macOS   | `/Users/Alice/Library/Application Support/Topiary/languages.ncl` |

This file is not automatically created by Topiary.

The next source is intended to be a project-specific settings file for
Topiary. When running Topiary in some directory, it will ascend the file
tree until it finds a `.topiary` directory. It will then read any `languages.ncl`
file present in that directory.

Finally, an explicit configuration file may be specified using the
`-C`/`--configuration` command line argument (or the
`TOPIARY_CONFIG_FILE` environment variable). This is intended for
driving Topiary under very specific use-cases.

The Topiary binary parses these sources in the following order.

1. The builtin configuration file.
2. The user configuration file in the OS's configuration directory.
3. The project specific Topiary configuration.
4. The explicit configuration file specified as a CLI argument.

### Configuration Options

The configuration file contains a record of languages. For instance, the one for
Nickel is defined as such:

```nickel
nickel = {
  extensions = ["ncl"],
},
```

The `name` field is used by Topiary to associate the language entry with the
query file and Tree-sitter grammar. This value should be written in lowercase.

The list of extensions is mandatory for every language, but does not necessarily
need to exist in every configuration file. It is sufficient if, for every
language, there is a single configuration file that defines the list of
extensions for that language.

A final optional field, called `indent`, exists to define the indentation method
for that language. Topiary defaults to two spaces `"  "` if it cannot find the
indent field in any configuration file for a specific language.

#### Specifying the grammar
Topiary can fetch and build the grammar for you, or a grammar can be provided by
some other method. To have Topiary fetch the grammar for you, specify the
`grammar.source.git` attribute of a language:
```nickel
nickel = {
  extensions = ["ncl"],
  grammar.source.git = {
    git = "https://github.com/nickel-lang/tree-sitter-nickel",
    rev = "43433d8477b24cd13acaac20a66deda49b7e2547",
  },
},
```

To specify a prebuilt grammar, specify the `grammar.source.path` attribute, which must point to a compiled grammar file:
```nickel
nickel = {
  extensions = ["ncl"],
  grammar.source.path = "/path/to/compiled/grammar/file.so",
},
```

> [!NOTE]
> If you want to link to a grammar file that has already been compiled
> by Topiary itself, those look like `<GIT_HASH>.so` (or the appropriate
> dynamic library extension for your platform).

For usage in Nix, a `languages_nix.ncl` file is provided that specifies the
paths of every language using the `@nickel@` syntax. These can easily be
replaced with nixpkgs' `substituteAll`.

### Overriding with `--merge-configuration`

By default, Topiary only considers the configuration file with the [highest priority](#configuration-sources). However, if the `-M` or `--merge-configuration` option is provided to the CLI, then all available configurations are merged together, as per the [Nickel specification](https://nickel-lang.org/user-manual/merging).

In that case, if one of the sources listed above attempts to define a language configuration
already present in the builtin configuration, or if two configuration files have conflicting values, then Topiary will display a Nickel error.

To understand why, one can read the [Nickel documentation on Merging](https://nickel-lang.org/user-manual/merging).
The short answer is that a priority must be defined. The builtin configuration
has everything defined with priority 0. Any priority above that will replace
any other priority. For example, to override the entire Bash configuration, use the following
Nickel file.

```nickel
{
  languages = {
    bash | priority 1 = {
      extensions = [ "sh" ],
      indent = "    ",
    },
  },
}
```

To override only the indentation, use the following Nickel file:
```nickel
{
  languages = {
    bash = {
      indent | priority 1 = "    ",
    },
  },
}
```

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
