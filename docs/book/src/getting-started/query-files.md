# Language query files

<!----------------------------------------------------------------------
TODO: Give an explanation/overview of Tree-sitter query files and how
Topiary uses them.

For a subsequent PR...

(See https://github.com/tweag/topiary/pull/750/files#r1961950658)
----------------------------------------------------------------------->

## Environment variables

Topiary needs to find the language query files (`*.scm`) to function
properly. By default, `topiary` looks for a `languages` directory in the
current working directory.

This won't work if you are running Topiary from a directory other than this
repository. In order to use Topiary without restriction, **you must set the
environment variable `TOPIARY_LANGUAGE_DIR` to point to the directory where
Topiary's language query files are located**.

By default, you should set it to `<local path of the topiary
repository>/topiary-queries/queries`, for example:

```sh
export TOPIARY_LANGUAGE_DIR=/home/me/tools/topiary/topiary-queries/queries
topiary format ./projects/helloworld/hello.ml
```

`TOPIARY_LANGUAGE_DIR` can alternatively be set at build time. Topiary
will pick the correspond path up and embed it into the `topiary` binary.
In that case, you don't have to worry about making
`TOPIARY_LANGUAGE_DIR` available at runtime any more. When
`TOPIARY_LANGUAGE_DIR` has been set at build time and is set at runtime
as well, the runtime value takes precedence.

<!----------------------------------------------------------------------
TODO: Move CONTRIBUTING.md into the Topiary Book

For a subsequent PR...
----------------------------------------------------------------------->
See [`CONTRIBUTING.md`](https://github.com/tweag/topiary/blob/main/CONTRIBUTING.md)
for details on setting up a development environment.
