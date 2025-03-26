# `mdbook-manmunge`

mdBook pre- and post-processor to help munge (a subset) of the Topiary
Book into manpages with [`mdbook-man`](https://github.com/vv9k/mdbook-man).

## Pre-processor

The binary provides the interface expected for mdBook to use as a
pre-processor. It can be added to your mdBook workflow (i.e.,
`book.toml`) with:

```toml
[preprocessor.manmunge]
```

## Post-processor

The binary also provides a post-processor, from `mdbook-man`'s rendered
manpage output (provided on standard input) and writing to standard
output. This is accessed with the `post-process` subcommand.
