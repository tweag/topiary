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

## Getting Started

### Installing

The project can be built and installed with Cargo from the repository
directory:

```bash
cargo install --path topiary-cli
```


## Suggested workflow

In order to work productively on query files, the following is one
suggested way to work:

1. Add a sample file to `topiary-cli/tests/samples/input`.

2. Copy the same file to `topiary-cli/tests/samples/expected`, and make any changes
   to how you want the output to be formatted.

3. If this is a new language, add its Tree-sitter grammar, extend
   `crate::language::Language` and process it everywhere, then make a
   mostly empty query file with just the `(#language!)` configuration.

4. Run:

   ```
   RUST_LOG=debug \
   cargo test -p topiary-cli \
              input_output_tester \
              -- --nocapture
   ```

   Provided it works, it should output a _lot_ of log messages. Copy
   that output to a text editor. You are particularly interested in the
   CST output that starts with a line like this: `CST node: {Node
   compilation_unit (0, 0) - (5942, 0)} - Named: true`.

   :bulb: As an alternative to using the debugging output, the
   `vis` visualisation subcommand line option exists to output the
   Tree-sitter syntax tree in a variety of formats.

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
Usage: ${PROGNAME} LANGUAGE [QUERY_FILE] [INPUT_SOURCE]

LANGUAGE can be one of the supported languages (e.g., "ocaml", "rust",
etc.). The packaged formatting queries for this language can be
overridden by specifying a QUERY_FILE.

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
[ormolu]: https://github.com/tweag/ormolu
[prettier]: https://prettier.io/
[rust]: https://www.rust-lang.org
[rustfmt]: https://rust-lang.github.io/rustfmt
[shfmt]: https://github.com/mvdan/sh
[toml]: https://toml.io
[topiary-playground]: https://topiary.tweag.io/playground
[topiary-website]: https://topiary.tweag.io
[tree-sitter-parsers]: https://tree-sitter.github.io/tree-sitter/#available-parsers
[tree-sitter-playground]: https://tree-sitter.github.io/tree-sitter/playground
[tree-sitter-query]: https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries
[tree-sitter]: https://tree-sitter.github.io/tree-sitter
[treefmt]: https://github.com/numtide/treefmt
