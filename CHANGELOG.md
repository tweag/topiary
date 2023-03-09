# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!----------------------------------------------------------------------
The "Unreleased" section should be amended as changes are merged
into main, using the below markdown as a template (using only sub-
headings as required). When a release is cut, these changes should
become retitled under the release version and date, linking to the
GitHub comparison, and a fresh "Unreleased" section should be started.
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
[unreleased]: https://github.com/tweag/topiary/compare/v0.1.0...HEAD

### Added
- Maintain a CHANGELOG and a documented release process.

### Changed
- Updated clap dependency to v4.1.

## [0.1.0] - 2023-03-09
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

## [0.0.1-prototype] - 2022-06-14
[0.0.1-prototype]: https://github.com/tweag/topiary/releases/tag/v0.0.1-prototype

This prototype release was created exclusively to show the validity of
the idea of using Tree-sitter to build a formatter. It includes only a
prototype JSON formatter.
