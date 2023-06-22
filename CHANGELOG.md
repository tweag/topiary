# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!----------------------------------------------------------------------
The "Unreleased" section should be amended as major changes are merged
into main, using the below markdown as a template (using only the sub-
headings required). When a release is cut, these changes should become
retitled under the release version and date, linking to the GitHub
comparison, and a fresh "Unreleased" section should be started.

Note that point releases (i.e., not patch releases) should also be given
a name, taking the form `ADJECTIVE TREE`, incrementing alphabetically.
This name should be decided amongst the team before the release.
------------------------------------------------------------------------

### Added
- <New feature>

### Changed
- <Changes in existing functionality>

### Deprecated
- <Soon-to-be removed features>

### Removed
- <Removed features>

### Fixed
- <Bug fixes>

### Security
- <Vulnerabilities>

----------------------------------------------------------------------->

## [Unreleased]
[unreleased]: https://github.com/tweag/topiary/compare/v0.2.3...HEAD
* [#526](https://github.com/tweag/topiary/pull/526) Multi-line comments can be indented properly using the new predicate @multi_line_indent_all.
* [#528](https://github.com/tweag/topiary/pull/528) Added a sample app and convenience functions for using the built-in queries.
* [#533](https://github.com/tweag/topiary/pull/533) Update tree-sitter-ocaml to 0.20.3

## [0.2.3] - 2023-06-20
[0.2.2]: https://github.com/tweag/topiary/compare/v0.2.2...v0.2.3

### Added
* [#513](https://github.com/tweag/topiary/pull/513) Added the `-t, --tolerate-
 parsing-errors` flags to Topiary, `tolerate_parsing_errors` to the `Format`
 operation of the library, and a "Tolerate parsing errors" checkmark to the
 playground. These options make Topiary ignore errors in the parsed file, and
 attempt to format it.
* [#506](https://github.com/tweag/topiary/pull/506) Allows the users to
 configure Topiary through a user-defined configuration file. More information
 can be found in the `README.md`.

### Changed
* [#523](https://github.com/tweag/topiary/pull/523) Skips rebuilding the tree-
  sitter `Query` when performing the idempotence check. This improves performance
  when not skipping the idempotence check by about `35%` for OCaml formatting.

### Removed
* [#508](https://github.com/tweag/topiary/pull/508) Simplified language
  detection by treating `ocaml` and `ocaml_interface` as two distinct languages.
  This ensures we only have one grammar per language. This
  removed the `-l ocaml_implementation` flag from Topiary and the
  `SupportedLanguage::OcamlImplementation` from the library.

### Fixed
* [#522](https://github.com/tweag/topiary/pull/522) Reverted the bump to the
 OCaml grammar and queries. This bump (for as of yet unknown reasons) had a
 catastrophic impact on Topiary's performance.

## [0.2.2] - 2023-06-12
[0.2.1]: https://github.com/tweag/topiary/compare/v0.2.1...v0.2.2

### Added
 * [#498](https://github.com/tweag/topiary/pull/498) Updated the playground to include a nicer editor.
 * [#487](https://github.com/tweag/topiary/pull/487) Added a flag to `format` function that allows skipping the idempotency check.
 * [#486](https://github.com/tweag/topiary/pull/486) Added the processing time to the online playground.
 * [#484](https://github.com/tweag/topiary/pull/484) Enabled the playground to perform on-the-fly formatting.
 * [#480](https://github.com/tweag/topiary/pull/480) Shows which languages are marked as experimental in the playground.

### Changed
 * [#490](https://github.com/tweag/topiary/pull/490) Bumped the Nickel grammar.
 * [#494](https://github.com/tweag/topiary/pull/494) Bumped the OCaml grammar, and fixed for the renamed `infix_operator` named node.

### Fixed
 * [#493](https://github.com/tweag/topiary/pull/493) Fixed
   [#492](https://github.com/tweag/topiary/issues/492) by only trimming newlines in prettyprinting.
 * [#491](https://github.com/tweag/topiary/pull/493) Fixed
   [#481](https://github.com/tweag/topiary/issues/492), a SIGSEGV in exhaustivity testing.

## [0.2.1] - 2023-05-23
[0.2.1]: https://github.com/tweag/topiary/compare/v0.2.0...v0.2.1

### Fixed
* Correctly bumped version number in `Cargo.toml`.

## [0.2.0]: Cyclic Cypress - 2023-05-22
[0.2.0]: https://github.com/tweag/topiary/compare/v0.1.0...v0.2.0

### Added
* Topiary [website](https://topiary.tweag.io), web-based
  [playground](https://topiary.tweag.io/playground) and logos.
* Full Nickel formatting support.
* Improved OCaml formatting support.
* `@append_antispace` and `@prepend_antispace` formatting capture names.
* WASM build target, for the web-based playground.
* Arbitrary whitespace indentation support.
* Exhaustive query checking in tests.
* Maintain a CHANGELOG and a documented release process.

### Changed
* Move to a build configuration file, rather than a mixture of
  hardcoding and parsing query predicates at runtime.
* Conditional predicates, in the query language, to reduce the number of
  formatting capture names.
* Higher fidelity exit codes.
* Idempotency check in terminal-based playground.
* Reduced verbosity of failed integration test output.
* Various improvements to the test suite.
* Idiomatic improvements to the Rust codebase.
* Restructured repository into a Cargo workspace.

### Fixed
* OCaml idempotency issues.
* Idempotency checking in integration tests.
* Don't process queries that match below leaf nodes.
* Skip over zero-byte matched nodes.

## [0.1.0]: Benevolent Beech - 2023-03-09
[0.1.0]: https://github.com/tweag/topiary/compare/v0.0.1-prototype...v0.1.0

This first public release focuses on the Topiary engine and providing
decent OCaml formatting support, with the formatting capture names
required to do so.

### Formatting Capture Names
* `@allow_blank_line_before`
* `@append_delimiter` / `@prepend_delimiter`
* `@append_multiline_delimiter` / `@prepend_multiline_delimiter`
* `@append_empty_softline` / `@prepend_empty_softline`
* `@append_hardline` / `@prepend_hardline`
* `@single_line_no_indent`
* `@append_indent_start` / `@prepend_indent_start`
* `@append_indent_end` / `@prepend_indent_end`
* `@append_input_softline` / `@prepend_input_softline`
* `@append_space` / `@prepend_space`
* `@append_spaced_softline` / `@prepend_spaced_softline`
* `@delete`
* `@singleline_delete`
* `@do_nothing`
* `@begin_scope` / `@end_scope` and scoped softlines

### Language Support
* OCaml (implementations and interfaces)
* JSON
* TOML
* Nickel (experimental)
* Bash (experimental)
* Tree-sitter query language (experimental)
* Rust (experimental)

### Miscellaneous
* Basic formatter authoring tools (terminal-based playground and tree visualisation)
* `pre-commit-hooks.nix` support

## [0.0.1-prototype]: Archetypal Aspen - 2022-06-14
[0.0.1-prototype]: https://github.com/tweag/topiary/releases/tag/v0.0.1-prototype

This prototype release was created exclusively to show the validity of
the idea of using Tree-sitter to build a formatter. It includes only a
prototype JSON formatter.
