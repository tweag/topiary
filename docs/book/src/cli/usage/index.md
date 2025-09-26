# Usage

The Topiary CLI uses a number of subcommands to delineate functionality.
These can be listed with `topiary --help`; each subcommand then has its
own, dedicated help text.

<!-- DO NOT REMOVE THE "usage:{start,end}" COMMENTS -->
<!-- usage:start -->
```
CLI app for Topiary, the universal code formatter.

Usage: topiary [OPTIONS] <COMMAND>

Commands:
  format         Format inputs
  visualise      Visualise the input's Tree-sitter parse tree
  config         Print the current configuration
  prefetch       Prefetch languages in the configuration
  coverage       Checks how much of the tree-sitter query is used
  check-grammar  Verifies if a given file is parseable by a given grammar
  completion     Generate shell completion script
  help           Print this message or the help of the given subcommand(s)

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -M, --merge-configuration            Enable merging for configuration files
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
  -V, --version                        Print version
```
<!-- usage:end -->

See the respective chapter for usage documentation on each subcommand:

- [`format`](format.md)
- [`visualise`](visualise.md)
- [`config`](config.md)
- [`completion`](completion.md)
- [`prefetch`](prefetch.md)
- [`coverage`](coverage.md)
- [`grammar-check`](check-grammar.md)

## Example

Once built, Topiary can be run like this:

```bash
echo '{"foo":"bar"}' | topiary format --language json
```

`topiary` can also be built and run from source via either Cargo or Nix,
if you have those installed:

```bash
echo '{"foo":"bar"}' | cargo run -- format --language json
echo '{"foo":"bar"}' | nix run . -- format --language json
```

This will output the following formatted code:

```json
{ "foo": "bar" }
```
