# Installation
Topiary can be installed in a few different ways. For more information on the
different ways, see the following pages:
  - [Package managers](./package-managers.md)
  - [Building from source](./building-from-source.md)
  - [Using with Nix](./using-with-nix.md)

Topiary needs to find the language query files (`.scm`) to function properly. By
default, `topiary` looks for a `languages` directory in the current working
directory.

This won't work if you are running Topiary from another directory than this
repository. In order to use Topiary without restriction, **you must set the
environment variable `TOPIARY_LANGUAGE_DIR` to point to the directory where
Topiary's language query files (`.scm`) are located**. By default, you should
set it to `<local path of the topiary repository>/topiary-queries/queries`, for example:

```sh
export TOPIARY_LANGUAGE_DIR=/home/me/tools/topiary/topiary-queries/queries
topiary fmt ./projects/helloworld/hello.ml
```

`TOPIARY_LANGUAGE_DIR` can alternatively be set at build time. Topiary will pick
the correspond path up and embed it into the `topiary` binary. In that case, you
don't have to worry about making `TOPIARY_LANGUAGE_DIR` available at run-time
anymore. When `TOPIARY_LANGUAGE_DIR` has been set at build time and is set at
run-time as well, the run-time value takes precedence.

See [`CONTRIBUTING.md`][contributing] for details on setting up a
development environment.
