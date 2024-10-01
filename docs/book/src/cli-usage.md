# CLI
### Usage
The Topiary CLI uses a number of subcommands to delineate functionality.
These can be listed with `topiary --help`; each subcommand then has its
own, dedicated help text.

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:ROOT -->
```
CLI app for Topiary, the universal code formatter.

Usage: topiary [OPTIONS] <COMMAND>

Commands:
  format      Format inputs
  visualise   Visualise the input's Tree-sitter parse tree
  config      Print the current configuration
  prefetch    Prefetch all languages in the configuration
  completion  Generate shell completion script
  help        Print this message or the help of the given subcommand(s)

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
  -V, --version                        Print version
```
<!-- usage:end:ROOT -->

#### Format

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:format -->
```
Format inputs

Usage: topiary format [OPTIONS] <--language <LANGUAGE>|FILES>

Arguments:
  [FILES]...
          Input files and directories (omit to read from stdin)

          Language detection and query selection is automatic, mapped from file extensions defined
          in the Topiary configuration.

Options:
  -t, --tolerate-parsing-errors
          Consume as much as possible in the presence of parsing errors

  -s, --skip-idempotence
          Do not check that formatting twice gives the same output

  -l, --language <LANGUAGE>
          Topiary language identifier (for formatting stdin)

  -q, --query <QUERY>
          Topiary query file override (when formatting stdin)

  -C, --configuration <CONFIGURATION>
          Configuration file

          [env: TOPIARY_CONFIG_FILE]

  -v, --verbose...
          Logging verbosity (increased per occurrence)

  -h, --help
          Print help (see a summary with '-h')
```
<!-- usage:end:format -->

When formatting inputs from disk, language selection is detected from
the input files' extensions. To format standard input, you must specify
the `--language` and, optionally, `--query` arguments, omitting any
input files.

Note: `fmt` is a recognised alias of the `format` subcommand.

#### Visualise

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:visualise -->
```
Visualise the input's Tree-sitter parse tree

Usage: topiary visualise [OPTIONS] <--language <LANGUAGE>|FILE>

Arguments:
  [FILE]
          Input file (omit to read from stdin)

          Language detection and query selection is automatic, mapped from file extensions defined
          in the Topiary configuration.

Options:
  -f, --format <FORMAT>
          Visualisation format

          [default: dot]

          Possible values:
          - dot:  GraphViz DOT serialisation
          - json: JSON serialisation

  -l, --language <LANGUAGE>
          Topiary language identifier (for formatting stdin)

  -q, --query <QUERY>
          Topiary query file override (when formatting stdin)

  -C, --configuration <CONFIGURATION>
          Configuration file

          [env: TOPIARY_CONFIG_FILE]

  -v, --verbose...
          Logging verbosity (increased per occurrence)

  -h, --help
          Print help (see a summary with '-h')
```
<!-- usage:end:visualise -->

When visualising inputs from disk, language selection is detected from
the input file's extension. To visualise standard input, you must
specify the `--language` and, optionally, `--query` arguments, omitting
the input file. The visualisation output is written to standard out.

Note: `vis`, `visualize` and `view` are recognised aliases of the
`visualise` subcommand.

#### Configuration

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:config -->
```
Print the current configuration

Usage: topiary config [OPTIONS]

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
```
<!-- usage:end:config -->

Note: `cfg` is a recognised alias of the `config` subcommand.

#### Shell Completion

Shell completion scripts for Topiary can be generated with the
`completion` subcommand. The output of which can be sourced into your
shell session or profile, as required.

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:completion -->
```
Generate shell completion script

Usage: topiary completion [OPTIONS] [SHELL]

Arguments:
  [SHELL]  Shell (omit to detect from the environment) [possible values: bash, elvish, fish,
           powershell, zsh]

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
```
<!-- usage:end:completion -->

For example, in Bash:

```bash
source <(topiary completion)
```

#### Prefetching

Topiary dynamically downloads, builds, and loads the tree-sitter grammars. In
order to ensure offline availability or speed up startup time, the grammars can
be prefetched and compiled.

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:prefetch-->
```
Prefetch all languages in the configuration

Usage: topiary prefetch [OPTIONS]

Options:
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
```
<!-- usage:end:prefetch -->

#### Logging

By default, the Topiary CLI will only output error messages. You can
increase the logging verbosity with a respective number of
`-v`/`--verbose` flags:

| Verbosity Flag | Logging Level           |
| :------------- | :---------------------- |
| None           | Errors                  |
| `-v`           | ...and warnings         |
| `-vv`          | ...and information      |
| `-vvv`         | ...and debugging output |
| `-vvvv`        | ...and tracing output   |

#### Exit Codes

The Topiary process will exit with a zero exit code upon successful
formatting. Otherwise, the following exit codes are defined:

| Reason                       | Code |
| :--------------------------- | ---: |
| Unspecified error            |    1 |
| CLI argument parsing error   |    2 |
| I/O error                    |    3 |
| Topiary query error          |    4 |
| Source parsing error         |    5 |
| Language detection error     |    6 |
| Idempotency error            |    7 |
| Unspecified formatting error |    8 |
| Multiple errors              |    9 |

When given multiple inputs, Topiary will do its best to process them
all, even in the presence of errors. Should _any_ errors occur, Topiary
will return a non-zero exit code. For more details on the nature of
these errors, run Topiary at the `warn` logging level (with `-v`).

#### Example

Once built, the program can be run like this:

```bash
echo '{"foo":"bar"}' | topiary fmt --language json
```

`topiary` can also be built and run from source via either Cargo or Nix,
if you have those installed:

```bash
echo '{"foo":"bar"}' | cargo run -- fmt --language json
echo '{"foo":"bar"}' | nix run . -- fmt --language json
```

It will output the following formatted code:

```json
{ "foo": "bar" }
```
