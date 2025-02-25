# Grammar prefetching

Topiary dynamically downloads, builds, and loads the Tree-sitter
grammars. In order to ensure offline availability or speed up start up
time, the grammars can be prefetched and compiled.

<!-- DO NOT REMOVE THE "usage" COMMENTS -->
<!-- usage:start:prefetch-->
```
Prefetch all languages in the configuration

Usage: topiary prefetch [OPTIONS]

Options:
  -f, --force                          Re-fetch existing grammars if they already exist
  -C, --configuration <CONFIGURATION>  Configuration file [env: TOPIARY_CONFIG_FILE]
  -M, --merge-configuration            Enable merging for configuration files
  -v, --verbose...                     Logging verbosity (increased per occurrence)
  -h, --help                           Print help
```
<!-- usage:end:prefetch -->
