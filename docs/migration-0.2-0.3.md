# Topiary CLI Migration Guide: v0.2 to v0.3

Full documentation for the CLI can be found in the project's
[`README`](/README.md). Herein we summarise how the v0.2.3 functionality
maps to the new interface introduced in v0.3.0, to aid migration.

## Formatting

### From Files, In Place

Before:
```
topiary [--skip-idempotence] \
        [--tolerate-parsing-errors] \
        --in-place \
        --input-files INPUT_FILES...
```

After:
```
topiary fmt [--skip-idempotence] \
            [--tolerate-parsing-errors] \
            INPUT_FILES...
```

### From File, To New File

Before:
```
topiary [--skip-idempotence] \
        [--tolerate-parsing-errors] \
        (--language LANGUAGE | --query QUERY) \
        --input-files INPUT_FILE \
        --output-file OUTPUT_FILE
```

After (use IO redirection):
```
topiary fmt [--skip-idempotence] \
            [--tolerate-parsing-errors] \
            (--language LANGUAGE [--query QUERY]) \
            < INPUT_FILE \
            > OUTPUT_FILE
```

### Involving Standard Input and Output

Before:
```
topiary [--skip-idempotence] \
        [--tolerate-parsing-errors] \
        (--language LANGUAGE [--query QUERY]) \
        (--input-files - | < INPUT_FILE) \
        [--output-file -]
```

After (use IO redirection):
```
topiary fmt [--skip-idempotence] \
            [--tolerate-parsing-errors] \
            (--language LANGUAGE [--query QUERY]) \
            < INPUT_FILE
```

## Visualisation

### From File

Before:
```
topiary --visualise[=FORMAT] \
        --input-files INPUT_FILE \
        [--output-file OUTPUT_FILE | > OUTPUT_FILE]
```

After:
```
topiary vis [--format FORMAT] \
            INPUT_FILE \
            [> OUTPUT_FILE]
```

### Involving Standard Input and Output

Before:
```
topiary --visualise[=FORMAT] \
        (--language LANGUAGE [--query QUERY]) \
        < INPUT_FILE \
        [--output-file OUTPUT_FILE | > OUTPUT_FILE]
```

After (use IO redirection):
```
topiary vis [--format FORMAT] \
            (--language LANGUAGE [--query QUERY]) \
            < INPUT_FILE \
            [> OUTPUT_FILE]
```

## Configuration

### Custom Configuration

To replicate the behaviour of v0.2.3, set the configuration collation
mode to `revise`. This can be done with the `TOPIARY_CONFIG_COLLATION`
environment variable, or the `--configuration-collation` argument.

The new default collation method is `merge`, which is subtly different
when it comes to collating collections.

### Overriding Configuration

Before (or using the `TOPIARY_CONFIGURATION_OVERRIDE` environment
variable):
```
topiary --configuration-override CONFIG_FILE ...
```

After (or using a combination of `TOPIARY_CONFIG_FILE` and
`TOPIARY_CONFIG_COLLATION` environment variables):
```
topiary --configuration CONFIG_FILE \
        --configuration-collation override \
        ...
```

### Examining Computed Configuration

Before (to standard error, as debug output, then proceeding with other
functions):
```
topiary --output-configuration ...
```

After (to standard output, in TOML format, as a dedicated function):
```
topiary cfg
```

## Logging

Before (via the `RUST_LOG` environment variable):
```
RUST_LOG=warn topiary ...
```

After (using `-v` command line flags):
```
topiary -v ...
```

The number of command line flags increases the verbosity:

| Verbosity Flag | `RUST_LOG` Equivalent |
| :------------- | :-------------------- |
| None           | `error` (default)     |
| `-v`           | `warn`                |
| `-vv`          | `info`                |
| `-vvv`         | `debug`               |
| `-vvvv`        | `trace`               |
