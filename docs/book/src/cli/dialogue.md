# Runtime dialogue

## Logging

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

## Exit codes

The Topiary process will exit with a zero exit code upon successful
formatting. Otherwise, the following exit codes are defined:

| Reason                       | Code |
| :--------------------------- | ---: |
| Negative result              |    1 |
| CLI argument parsing error   |    2 |
| I/O error                    |    3 |
| Topiary query error          |    4 |
| Source parsing error         |    5 |
| Language detection error     |    6 |
| Idempotency error            |    7 |
| Unspecified formatting error |    8 |
| Multiple errors              |    9 |
| Unspecified error            |   10 |

Negative results with error code `1` only happen when Topiary is called
with the `coverage` sub-command, if the input does not cover 100% of the
query.

When given multiple inputs, Topiary will do its best to process them
all, even in the presence of errors. Should _any_ errors occur, Topiary
will return a non-zero exit code. For more details on the nature of
these errors, run Topiary at the `warn` logging level (with `-v`).
