# Grammar checking

This subcomand checks that one or more inputs are parseable to a tree-sitter grammar,
returning all locations where a parsing error occurred.

<!-- DO NOT REMOVE THE "usage:{start,end}" COMMENTS -->
<!-- usage:start -->
```
Verifies if a given file is parseable by a given grammar

Usage: topiary check-grammar [OPTIONS] <--language <LANGUAGE>|FILES>

Arguments:
  [FILES]...  Input files and directories (omit to read from stdin)

Options:
  -l, --language <LANGUAGE>            Topiary language identifier (when formatting stdin)
  -q, --query <QUERY>                  Topiary query file override (when formatting stdin)
  -L, --follow-symlinks                Follow symlinks (when formatting files)
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -M, --merge-configuration            Enable merging for configuration files
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help (see more with '--help')
```
<!-- usage:end -->
