# Style guide

This is not part of the Topiary Book, but exists to try to document a
consistent style for future changes. These are not intended to be
prescriptive rules, but exist as guidelines to keep the Topiary Book
maintained.

The following are in alphabetic order:

## Capture names

The Tree-sitter query syntax `@foo` is a **"capture name"**, or
"capture" where the context makes the elision obvious.

## English

The Topiary Book is mostly written in British English, but this dialect
is not mandated; American or other dialects of English are perfectly
acceptable and can be intermixed.

## Formatting formatting queries

Formatting queries ought to be formatted in the same way as Topiary
formats them (with the `tree_sitter_query` formatting queries). As of
writing, this is Algol-style indentation, rather than Lisp-style.

(This may change; see issue #828.)

## Formatting queries

The queries used to define formatting rules should be referred to as
**"formatting queries"**. Using "rules" or "styles", etc. can confuse
what it Topiary's core mechanism.

## Inline code formatting

Use inline code formatting, with backticks, only for code and
code-adjacent text. For example, `topiary` would specifically be the
Topiary CLI binary, rather than Topiary as a concept.

## Line wrapping

The Topiary Book's markdown files are mostly wrapped to 72 columns. This
is not strictly necessary, but it aids reading in a terminal emulator.

## Nix

**"Nix"** is capitalised (unless it is referring to the binary, in which
case it's `nix`). All other Nix-related concepts should be treated as
normal words (i.e., lowercase, unless they start a sentence). For
example: **"nixpkgs"**, **"flake"**, **"devshell"**, etc.

## Notes and warnings

Notes should be marked up like so:

> **Note**\
> Here is the note text.

Warnings should use mdbook's `warning` class:

```markdown
<div class="warning">

Here is the warning text.

<div>
```

## Single and multi-line

Use **"single-line"** and **"multi-line"** (i.e., hyphenated), rather
than spaced or juxtaposed.

## Title case

The Topiary Book uses sentence case, rather than title case, for
headings.

## Tree-sitter

It's **"Tree-sitter"**, not "Tree Sitter" or "tree-sitter", etc.

## Tree-sitter grammars

The Topiary Book uses **"Tree-sitter grammar"**, rather than
"Tree-sitter parser". "Tree-sitter" can be omitted where the context
makes the elision obvious.

## Unordered lists

Use `-` as the unordered list markup, rather than `*`.
