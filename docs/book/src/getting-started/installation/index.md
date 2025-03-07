# Installation

Topiary can be installed in a few different ways. For more information
on these, see the respective chapter:

- [Package managers](package-managers.md)
- [Building from source](building-from-source.md)
- [Using Nix](using-nix.md)

## Dependencies

The Topiary CLI will build Tree-sitter grammars on demand, optionally
fetching them from a Git remote first. For this to work, your
environment will need a C/C++ toolchain (i.e., compiler and linker)
available. If this is available, it should "just work"; otherwise, refer
to the [underlying mechanism's documentation](https://docs.rs/cc/latest/cc/#external-configuration-via-environment-variables)
for configuration advice.

Alternatively, the [Tree-sitter CLI](https://github.com/tree-sitter/tree-sitter/blob/master/cli/README.md)
can be used to build Tree-sitter grammars outside of Topiary, which can
then be loaded through [configuration](../../cli/configuration.md#specifying-the-grammar)
from your local filesystem.
