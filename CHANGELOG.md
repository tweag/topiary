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
[unreleased]: https://github.com/tweag/topiary/compare/v0.2.1...HEAD

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
