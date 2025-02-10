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

[Full list of changes](https://github.com/tweag/topiary/compare/v0.6.0...HEAD)


<!-- ### Added -->
<!-- - <New feature> -->

### Changed
<!-- - <Changes in existing functionality> -->
- [#859](https://github.com/tweag/topiary/pull/859) Break up integration tests per language, thanks to @mkatychev

<!-- ### Deprecated -->
<!-- - <Soon-to-be removed features> -->

<!-- ### Removed -->
<!-- - <Removed features> -->

### Fixed
<!-- - <Bug fixes> -->
- [#867](https://github.com/tweag/topiary/pull/867) Enable coverage check and add code samples for OpenSCAD
- [#869](https://github.com/tweag/topiary/pull/869) Disable parallel grammar building on Windows

<!-- ### Security -->
<!-- - <Vulnerabilities> -->

## v0.6.0 - Gilded Ginkgo - 2025-01-30

[Full list of changes](https://github.com/tweag/topiary/compare/v0.5.1...v0.6.0)

### Added
- [#747](https://github.com/tweag/topiary/pull/747) Added support for specifying paths to prebuilt grammars in Topiary's configuration
- [#785](https://github.com/tweag/topiary/pull/785) Added the `coverage` command, that checks how much of the query file is used by the input
- [#786](https://github.com/tweag/topiary/pull/786) Re-introduce tests to check that all of the language queries are useful
- [#832](https://github.com/tweag/topiary/pull/832) Added `typos-cli` configuration to workspace `Cargo.toml` for spellchecking, thanks to @mkatychev
- [#838](https://github.com/tweag/topiary/pull/838) Added `@upper_case` and `@lower_case` captures, thanks to @ctdunc
- [#845](https://github.com/tweag/topiary/pull/747) Added support for OpenSCAD, thanks to @mkatychev
- [#851](https://github.com/tweag/topiary/pull/851) Added support for following symlinks when specifying input files for formatting

### Changed
- [#780](https://github.com/tweag/topiary/pull/780) Measuring scopes are now independent from captures order
- [#790](https://github.com/tweag/topiary/pull/790) No longer merge config files by default, use priority instead
- [#794](https://github.com/tweag/topiary/pull/794) Bump the `tree-sitter` dependency to 0.24, thanks to @ZedThree
- [#801](https://github.com/tweag/topiary/pull/801) Improved documentation of the `visualise` subcommand
- [#811](https://github.com/tweag/topiary/pull/811) The `config` subcommand now outputs a Nickel file instead of some inner representation
- [#830](https://github.com/tweag/topiary/pull/830) Use `tree-sitter-loader` to build grammars, rather than rolling our own

### Fixed
- [#779](https://github.com/tweag/topiary/pull/779) Load relevant grammars before CLI tests
- [#799](https://github.com/tweag/topiary/pull/799) Line break after table-less pairs in TOML
- [#813](https://github.com/tweag/topiary/pull/813) In-place writing on Windows (also introduced a minimal Windows CI)
- [#822](https://github.com/tweag/topiary/pull/822) Various Bash fixes and improvements
- [#826](https://github.com/tweag/topiary/pull/826) Various Tree-sitter query fixes, thanks to @mkatychev
- [#853](https://github.com/tweag/topiary/pull/853) Small fixes to CLI logging and IO

## v0.5.1 - Fragrant Frangipani - 2024-10-22

[Full list of changes](https://github.com/tweag/topiary/compare/v0.4.0...v0.5.1)

### Added
- [#705](https://github.com/tweag/topiary/pull/705) Added support for Nickel 1.7 extended pattern formatting
- [#737](https://github.com/tweag/topiary/pull/737) Added the `prefetch` command, that prefetches and caches all grammars in the current configuration
- [#755](https://github.com/tweag/topiary/pull/755) Introduce measuring scopes, which can be used in conjunction with regular scopes to add expressivity to the formatting queries.
- [#760](https://github.com/tweag/topiary/pull/760) Introduce optional `query_name` predicate, to help with query logging and debugging.

### Fixed
- [#720](https://github.com/tweag/topiary/pull/720) [#722](https://github.com/tweag/topiary/pull/722) [#723](https://github.com/tweag/topiary/pull/723) [#724](https://github.com/tweag/topiary/pull/724) [#735](https://github.com/tweag/topiary/pull/735) [#738](https://github.com/tweag/topiary/pull/738) [#739](https://github.com/tweag/topiary/pull/739) [#745](https://github.com/tweag/topiary/pull/745) [#755](https://github.com/tweag/topiary/pull/755) [#759](https://github.com/tweag/topiary/pull/759) [#764](https://github.com/tweag/topiary/pull/764 Various OCaml improvements
- [#762](https://github.com/tweag/topiary/pull/762) Various Rust improvements
- [#744](https://github.com/tweag/topiary/pull/744) [#768](https://github.com/tweag/topiary/pull/768) Nickel: fix the formatting of annotated multiline let-bindings
- [#763](https://github.com/tweag/topiary/pull/763) Various Bash fixes and improvements
- [#761](https://github.com/tweag/topiary/pull/761) No longer use error code 1 for unspecified errors
- [#770](https://github.com/tweag/topiary/pull/770) Fallback to compile-time included query files when formatting a file

### Changed
- [#704](https://github.com/tweag/topiary/pull/704) Refactors our postprocessing code to be more versatile.
- [#711](https://github.com/tweag/topiary/pull/711) Feature gate all grammars, with supported and contributed languages built by default.
- [#716](https://github.com/tweag/topiary/pull/716) Dynamically fetch, compile, and load language grammars. Topiary now no longer ships with statically linked grammars.
- [#732](https://github.com/tweag/topiary/pull/732) Change how function application and parenthesized expressions are treated in Nickel to reduce the overall noise and indentation
- [#736](https://github.com/tweag/topiary/pull/668) Updates our Nickel grammar, and adds support for let blocks.
- [#769](https://github.com/tweag/topiary/pull/769) Move the web playground to a separate branch
- [#773](https://github.com/tweag/topiary/pull/773) Change the status of Bash from "experimental" to "supported"

## v0.4.0 - Exquisite Elm - 2024-05-15

[Full list of changes](https://github.com/tweag/topiary/compare/v0.3.0...v0.4.0)

### Added
- [#589](https://github.com/tweag/topiary/pull/589) Added syntax highlighting to the playground (excluding Nickel)
- [#686](https://github.com/tweag/topiary/pull/686) Added support for Nickel pattern formatting
- [#697](https://github.com/tweag/topiary/pull/697) Setting the log level to INFO now outputs the pattern locations in a (row, column) way.
- [#699](https://github.com/tweag/topiary/pull/699) Added support for CSS, thanks to @lavigneer
- [#703](https://github.com/tweag/topiary/pull/703) Switched our configuration over to Nickel

### Fixed
- [#626](https://github.com/tweag/topiary/pull/626) [#627](https://github.com/tweag/topiary/pull/627) [#628](https://github.com/tweag/topiary/pull/628) [#626](https://github.com/tweag/topiary/pull/648) Various OCaml improvements
- [#673](https://github.com/tweag/topiary/pull/673) Various TOML fixes
- [#678](https://github.com/tweag/topiary/pull/678) Ensures the client example project builds, and is tested in CI
- [#677](https://github.com/tweag/topiary/pull/677) Ensures our playground builds consistently in CI
- [#682](https://github.com/tweag/topiary/pull/682) Removes prepended linebreaks from equal signs in Nickel annotations
- [#692](https://github.com/tweag/topiary/pull/692) Improves our installation instructions, thanks to @Jasha10

### Changed
- [#664](https://github.com/tweag/topiary/pull/664) Ensures source positions in the logs are consistent thanks to @evertedsphere
- [#668](https://github.com/tweag/topiary/pull/668) Updates our Nickel grammar
- [#672](https://github.com/tweag/topiary/pull/672) Completely refactors our crate layout, preparing for a release on crates.io

## v0.3.0 - Dreamy Dracaena - 2023-09-22

[Full list of changes](https://github.com/tweag/topiary/compare/v0.2.3...v0.3.0)

### Added
* [#538](https://github.com/tweag/topiary/pull/538) Using `cargo-dist` to release Topiary binaries.
* [#528](https://github.com/tweag/topiary/pull/528) [#609](https://github.com/tweag/topiary/pull/609) Created a `topiary-queries` crate that exports the builtin query files.
* [#526](https://github.com/tweag/topiary/pull/526) Multi-line comments can be indented properly using the new predicate @multi_line_indent_all.
* [#533](https://github.com/tweag/topiary/pull/533) Topiary can now process multiple files with one call.
* [#553](https://github.com/tweag/topiary/pull/553) In Nickel, indent when a new infix_expr chain starts.
* [#557](https://github.com/tweag/topiary/pull/557) Topiary now falls back to the buildin queries when no other query files could be located.
* [#573](https://github.com/tweag/topiary/pull/573) Added OCamllex support (without injections).
* [#576](https://github.com/tweag/topiary/pull/576) Added append/prepend versions of scope captures.

### Changed
* [#535](https://github.com/tweag/topiary/pull/535) Improved error message when idempotency fails due to invalid output in the first pass.
* [#576](https://github.com/tweag/topiary/pull/576) Allows prepending/appending `@begin_scope` and `@end_scope`
* [#583](https://github.com/tweag/topiary/pull/583) Modernisation of the command line interface (see [the CLI Migration Guide](/docs/migration-0.2-0.3.md), for details)
* [#535](https://github.com/tweag/topiary/pull/535) Change the error message for an idempotency error to be more descriptive.
* [#536](https://github.com/tweag/topiary/pull/536) [#537](https://github.com/tweag/topiary/pull/537) [#578](https://github.com/tweag/topiary/pull/578) [#626](https://github.com/tweag/topiary/pull/626) [#627](https://github.com/tweag/topiary/pull/627) [#628](https://github.com/tweag/topiary/pull/628) Various OCaml improvements.
* [#623](https://github.com/tweag/topiary/pull/623) [#624](https://github.com/tweag/topiary/pull/624) [#625](https://github.com/tweag/topiary/pull/625) Various Toml improvements thanks @pjjw.

### Fixed
* [#533](https://github.com/tweag/topiary/pull/533) Bump tree-sitter-ocaml version, which allowed reintroduction of some removed queries.
* [#550](https://github.com/tweag/topiary/pull/550) Fixed handling of antispace in post-processing.
* [#552](https://github.com/tweag/topiary/pull/552) Fixed Nickel tag removal.
* [#554](https://github.com/tweag/topiary/pull/554) [#555](https://github.com/tweag/topiary/pull/555) Fixed Nickel idempotency issue related to annotations.
* [#565](https://github.com/tweag/topiary/pull/565) Fixed an issue where Topiary would remove whitespace between predicate parameters in query files.

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
