# Language Support
<!-- Update this section as necessary on new developments/releases -->
Topiary's support of languages comes in two levels of maturity:
supported and experimental.

#### Supported

These formatting styles cover their target language and fulfill Topiary's
stated design goals. They are exposed, in Topiary, through the
`--language` command line flag, or language detection (based on file
extension).

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

#### Experimental

These languages' formatting styles are subject to change and/or not yet
considered production-ready. They are _not_ built by default and are
gated behind a feature flag (either `experimental`, for all of them, or
by their individual name). Once included, they can be accessed in
Topiary in the usual way.

* [Bash]
* [Rust]
