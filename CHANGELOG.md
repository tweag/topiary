# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and [`cargo-dist`'s expected
format](https://opensource.axo.dev/cargo-dist/book/simple-guide.html#release-notes),
and this project adheres to [Semantic
Versioning](https://semver.org/spec/v2.0.0.html).

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

## Unreleased

[Full list of changes](https://github.com/tweag/topiary/compare/v0.2.3...HEAD)

### Added
* [#538](https://github.com/tweag/topiary/pull/538) Using `cargo-dist` to release Topiary binaries.
* [#528](https://github.com/tweag/topiary/pull/528) Added a sample app and convenience functions for using the built-in queries.
* [#526](https://github.com/tweag/topiary/pull/526) Multi-line comments can be indented properly using the new predicate @multi_line_indent_all.

### Changed
* [#535](https://github.com/tweag/topiary/pull/535) Improved error message when idempotency fails due to invalid output in the first pass.
* [#533](https://github.com/tweag/topiary/pull/533) Update tree-sitter-ocaml to 0.20.3
* [#576](https://github.com/tweag/topiary/pull/576) Allows prepending/appending `@begin_scope` and `@end_scope`
* [#583](https://github.com/tweag/topiary/pull/583) Modernisation of the command line interface (see [below](#cli-migration-guide), for details)

#### CLI Migration Guide

Full documentation for the CLI can be found in the project's
[`README`](/README.md). Herein we summarise how the v0.2.3 functionality
maps to the new interface, to aid migration.

##### Formatting

###### From Files, In Place

Before:
```
topiary [--skip-idempotence] \
        [--tolerate-parsing-errors] \
        --in-place \
        --input-files INPUT_FILES...
```

After:
```
topiary fmt [--skip-idempotence] \
            [--tolerate-parsing-errors] \
            INPUT_FILES...
```

###### From File, To New File

Before:
```
topiary [--skip-idempotence] \
        [--tolerate-parsing-errors] \
        (--langauge LANGUAGE | --query QUERY) \
        --input-files INPUT_FILE \
        --output-file OUTPUT_FILE
```

After (use IO redirection):
```
topiary [--skip-idempotence] \
        [--tolerate-parsing-errors] \
        (--langauge LANGUAGE | --query QUERY) \
        < INPUT_FILE \
        > OUTPUT_FILE
```

###### Involving Standard Input and Output

Before:
```
topariy [--skip-idempotence] \
        [--tolerate-parsing-errors] \
        (--langauge LANGUAGE | --query QUERY) \
        (--input-files - | < INPUT_FILE) \
        [--output-file -]
```

After (use IO redirection):
```
topiary [--skip-idempotence] \
        [--tolerate-parsing-errors] \
        (--langauge LANGUAGE | --query QUERY) \
        < INPUT_FILE
```

##### Visualisation

###### From File

Before:
```
topiary --visualise[=FORMAT] \
        --input-files INPUT_FILE \
        [--output-file OUTPUT_FILE | > OUTPUT_FILE]
```

After:
```
topiary vis [--format FORMAT] \
            INPUT_FILE \
            [> OUTPUT_FILE]
```

###### Involving Standard Input and Output

Before:
```
topiary --visualise[=FORMAT] \
        (--langauge LANGUAGE | --query QUERY) \
        < INPUT_FILE \
        [--output-file OUTPUT_FILE | > OUTPUT_FILE]
```

After (use IO redirection):
```
topiary vis [--format FORMAT] \
            (--langauge LANGUAGE | --query QUERY) \
            < INPUT_FILE \
            [> OUTPUT_FILE]
```

##### Configuration

###### Custom Configuration

To replicate the behaviour of v0.2.3, set the configuration collation
mode to `revise`. This can be done with the `TOPIARY_CONFIG_COLLATION`
environment variable, or the `--configuration-collation` argument.

The new default collation method is `merge`, which is subtly different
when it comes to collating collections.

###### Overriding Configuration

Before (or using the `TOPIARY_CONFIGURATION_OVERRIDE` environment
variable):
```
topiary --configuration-override CONFIG_FILE ...
```

After (or using a combination of `TOPIARY_CONFIG_FILE` and
`TOPIARY_CONFIG_COLLATION` environment variables):
```
topiary --configuration CONFIG_FILE \
        --configuration-collation override \
        ...
```

###### Examining Computed Configuration

Before (to standard error, then proceeding with other functions):
```
topiary --output-configuration ...
```

After (to standard output, as a dedicated function):
```
topiary cfg
```

## v0.2.3 - Cyclic Cypress - 2023-06-20

[Full list of changes](https://github.com/tweag/topiary/compare/v0.2.2...v0.2.3)

### Added
* [#513](https://github.com/tweag/topiary/pull/513) Added the `-t, --tolerate-parsing-errors` flags to Topiary, `tolerate_parsing_errors` to the `Format` operation of the library, and a "Tolerate parsing errors" checkmark to the playground. These options make Topiary ignore errors in the parsed file, and attempt to format it.
* [#506](https://github.com/tweag/topiary/pull/506) Allows the users to configure Topiary through a user-defined configuration file. More information can be found in the `README.md`.

### Changed
* [#523](https://github.com/tweag/topiary/pull/523) Skips rebuilding the tree-sitter `Query` when performing the idempotence check. This improves performance when not skipping the idempotence check by about `35%` for OCaml formatting.

### Removed
* [#508](https://github.com/tweag/topiary/pull/508) Simplified language detection by treating `ocaml` and `ocaml_interface` as two distinct languages. This ensures we only have one grammar per language. This removed the `-l ocaml_implementation` flag from Topiary and the `SupportedLanguage::OcamlImplementation` from the library.

### Fixed
* [#522](https://github.com/tweag/topiary/pull/522) Reverted the bump to the OCaml grammar and queries. This bump (for as of yet unknown reasons) had a catastrophic impact on Topiary's performance.

## v0.2.2 - Cyclic Cypress - 2023-06-12

[Full list of changes](https://github.com/tweag/topiary/compare/v0.2.1...v0.2.2)

### Added
 * [#498](https://github.com/tweag/topiary/pull/498) Updated the playground to include a nicer editor.
 * [#487](https://github.com/tweag/topiary/pull/487) Added a flag to `format` function that allows skipping the idempotency check.
 * [#486](https://github.com/tweag/topiary/pull/486) Added the processing time to the online playground.
 * [#484](https://github.com/tweag/topiary/pull/484) Enabled the playground to perform on-the-fly formatting.
 * [#480](https://github.com/tweag/topiary/pull/480) Shows which languages are marked as experimental in the playground.

### Changed
 * [#494](https://github.com/tweag/topiary/pull/494) Bumped the OCaml grammar, and fixed for the renamed `infix_operator` named node.
 * [#490](https://github.com/tweag/topiary/pull/490) Bumped the Nickel grammar.

### Fixed
 * [#493](https://github.com/tweag/topiary/pull/493) Fixed [#492](https://github.com/tweag/topiary/issues/492) by only trimming newlines in prettyprinting.
 * [#491](https://github.com/tweag/topiary/pull/493) Fixed [#481](https://github.com/tweag/topiary/issues/492), a SIGSEGV in exhaustivity testing.

## v0.2.1 - Cyclic Cypress - 2023-05-23

[Full list of changes](https://github.com/tweag/topiary/compare/v0.2.0...v0.2.1)

### Fixed
* Correctly bumped version number in `Cargo.toml`.

## v0.2.0 - Cyclic Cypress - 2023-05-22

[Full list of changes](https://github.com/tweag/topiary/compare/v0.1.0...v0.2.0)

### Added
* Topiary [website](https://topiary.tweag.io), web-based [playground](https://topiary.tweag.io/playground) and logos.
* Full Nickel formatting support.
* Improved OCaml formatting support.
* `@append_antispace` and `@prepend_antispace` formatting capture names.
* WASM build target, for the web-based playground.
* Arbitrary whitespace indentation support.
* Exhaustive query checking in tests.
* Maintain a CHANGELOG and a documented release process.

### Changed
* Move to a build configuration file, rather than a mixture of hardcoding and parsing query predicates at runtime.
* Conditional predicates, in the query language, to reduce the number of formatting capture names.
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

## v0.1.0 - Benevolent Beech - 2023-03-09

[Full list of changes](https://github.com/tweag/topiary/compare/v0.0.1-prototype...v0.1.0)

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

## v0.0.1-prototype - Archetypal Aspen - 2022-06-14

[Full list of changes](https://github.com/tweag/topiary/compare/03e1fc8...v0.0.1-prototype)

This prototype release was created exclusively to show the validity of the idea of using Tree-sitter to build a formatter. It includes only a prototype JSON formatter.
