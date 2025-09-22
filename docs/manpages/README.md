# `man topiary`

man pages for Topiary are generated from a subset of the [Topiary
Book](../book); specifically the Topiary CLI chapters. The same
mechanism is used to create man pages as the Topiary Book: [mdBook],
using the [`mdbook-man`][mdbook-man] renderer and a pre- and
post-processor written by the Topiary team.

## Getting started

> [!IMPORTANT]
> [mdBook (`mdbook`)][mdbook], `mdbook-man` and
> [`mdbook-manmunge`][mdbook-manmunge] must be in your `$PATH`. See the
> respective links for installation instructions, _except_ for
> `mdbook-man`. This must be installed from GitHub as the latest release
> (0.1) is not sufficient:
>
> ```console
> $ cargo install --git https://github.com/vv9k/mdbook-man.git
> ```

To build the man pages and install, run:

```console
$ make
$ sudo make install
```

By default the man pages will be installed to
`/usr/share/man/man1/topiary.1.gz`. The `/usr/share/man` prefix can be
overridden by setting the `MAN_DIR` variable. For example:

```console
$ make
$ sudo make install MAN_DIR=/opt/topiary/share/man
```

### Using Nix

The Topiary Nix devshell provides mdBook and `mdbook-manmunge`, leaving
you to build and install `mdbook-manmunge` (see [below][mdbook-manmunge]).

Alternatively, a package exists that will build the man pages as its
derivation output, handling all the dependencies for you. For example:

```console
$ nix build .#topiary-manpages
```

## `mdbook-manmunge`

The pre- and post-processor has been designed to mould certain aspects
of the output of `mdbook-man` into a more palatable form. This is
arguably the wrong approach; a better solution would have been to fork
`mdbook-man` and make the necessary changes there. This may happen in
the future, but this solution will do for now.

### Installation

`mdbook-manmunge` is not available on [crates.io](https://crates.io).
However, it can be built and installed locally from this repository:

```console
$ cargo install --path mdbook-manmunge
```

> [!TIP]
> You will need to include Cargo's binary directory (e.g.,
> `~/.cargo/bin`) in your `$PATH`.

> [!NOTE]
> `mdbook-manmunge` _may_ complain about different versions of mdBook
> (i.e., the caller vs. its internal dependency) when it is run.
> Anecdotally, this can be ignored for now, however breaking changes may
> be introduced in the future.

<!-- Links -->
[mdbook]: https://rust-lang.github.io/mdBook
[mdbook-man]: https://github.com/vv9k/mdbook-man
[mdbook-manmunge]: #mdbook-manmunge
