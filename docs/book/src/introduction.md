# Introduction
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

