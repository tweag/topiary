# Terminal-based playground

Nix users may also find the `playground` script to be helpful in aiding
the interactive development of query files. When run in a terminal,
inside the devshell defined in the Nix flake, it will format the given
source input with the requested query file, updating the output on any
inotify event against those files.

```
Usage: playground LANGUAGE [QUERY_FILE] [INPUT_SOURCE]

LANGUAGE can be one of the supported languages (e.g., "ocaml", "rust",
etc.). The packaged formatting queries for this language can be
overridden by specifying a QUERY_FILE.

The INPUT_SOURCE is optional. If not specified, it defaults to trying
to find the bundled integration test input file for the given language.
```

For example, the playground can be run in a terminal emulator pane, with
your editor of choice open in another.

<div class="warning">
The use of inotify limits this tool to Linux systems, only.
</div>
