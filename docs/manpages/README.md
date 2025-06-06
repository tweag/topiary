# `man topiary`

man pages for Topiary are generated from a subset of the [Topiary
Book](../book); specifically the Topiary CLI chapters. The same
mechanism is used to create man pages as the Topiary Book:
[mdBook](https://rust-lang.github.io/mdBook), using the
[`mdbook-man`](https://github.com/vv9k/mdbook-man) renderer and a pre-
and post-processor written by the Topiary team.

## Getting started

To build the man pages and install, run:

```console
$ make
$ sudo make install
```

By default the man pages will be installed to
`/usr/share/man/man1/topiary.1.gz`. The `/usr/share/man` prefix can be
overridden by setting the `PREFIX` variable. For example:

```console
$ make
$ sudo make install PREFIX=/opt/topiary/share/man
```

> [!NOTE]
> The `Makefile` will build the pre- and post-processor with Cargo,
> which manages its own dependencies, etc.

## `mdbook-manmunge`

The pre- and post-processor has been designed to mould certain aspects
of the output of `mdbook-man` into a more palatable form. This is
arguably the wrong approach; a better solution would have been to fork
`mdbook-man` and make the necessary changes there. This may happen in
the future, but this solution will do for now.
