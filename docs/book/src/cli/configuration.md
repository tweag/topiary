# Configuration

Topiary is configured using `languages.ncl` files. The `.ncl` extension
relates to [Nickel](https://nickel-lang.org/), a configuration language
created by Tweag. There are up to four sources where Topiary checks for
such a file.

## Configuration sources

At build time the [`languages.ncl`](https://github.com/tweag/topiary/blob/main/topiary-config/languages.ncl)
in the root of the Topiary repository is embedded into Topiary. This
file is parsed at runtime. The purpose of this `languages.ncl` file is
to provide sane defaults for users of Topiary (both the library and the
CLI binary).

The next two are read by the Topiary binary at runtime and allow the
user to configure Topiary to their needs. The first is intended to be
user specific, and can thus be found in the configuration directory of
the OS:

| OS      | Typical Configuration Path                                       |
| :------ | :--------------------------------------------------------------- |
| Unix    | `/home/alice/.config/topiary/languages.ncl`                      |
| macOS   | `/Users/Alice/Library/Application Support/Topiary/languages.ncl` |
| Windows | `C:\Users\Alice\AppData\Roaming\Topiary\config\languages.ncl`    |

This file is not automatically created by Topiary.

The next source is intended to be a project-specific settings file for
Topiary. When running Topiary in some directory, it will ascend the file
tree until it finds a `.topiary` directory. It will then read any
`languages.ncl` file present in that directory.

Finally, an explicit configuration file may be specified using the
`-C`/`--configuration` command line argument (or the
`TOPIARY_CONFIG_FILE` environment variable). This is intended for
driving Topiary under very specific use-cases.

To summarise, Topiary consumes configuration from these sources in the
following order (highest to lowest):

1. The explicit configuration file specified as a CLI argument.
2. The project-specific Topiary configuration.
3. The user configuration file in the OS's configuration directory.
4. The built-in configuration file.

### Configuration merging

By default, Topiary only considers the configuration file with the
highest priority. However, if the `-M`/`--merge-configuration` option is
provided to the CLI, then all available configurations are merged
together, as per the [Nickel specification](https://nickel-lang.org/user-manual/merging).

In which case, if one of the sources listed above attempts to define a
language configuration already present in the built-in configuration, or
if two configuration files have conflicting values, then Topiary will
display a Nickel error.

To understand why, one can read the [Nickel documentation on
merging](https://nickel-lang.org/user-manual/merging). However, the
short answer is that a priority must be defined. The built-in
configuration has everything defined with priority 0. Any priority above
that will replace any other priority. For example, to override the
entire Bash configuration, use the following Nickel file.

```nickel
{
  languages = {
    # Alternatively, use `priority 1`, rather than `force`
    bash | force = {
      extensions = [ "sh" ],
      indent = "    ",
    },
  },
}
```

To override only the indentation, use the following Nickel file:

```nickel
{
  languages = {
    bash = {
      indent | force = "    ",
    },
  },
}
```

<div class="warning">

The merging semantics for Topiary's grammar configuration (see
[below](#specifying-the-grammar)) is not yet fully defined; see issue
[#861](https://github.com/tweag/topiary/issues/861).

</div>

## Configuration options

The configuration file contains a record of languages. That is, it
defines language identifiers and configures them accordingly.

For instance, the configuration for Nickel is defined as such:

```nickel
nickel = {
  extensions = ["ncl"],
},
```

The language identifier is used by Topiary to associate the language
entry with the respective query file, as well exposing it to the user
through the `--language` CLI argument. This value should be written in
lowercase.

### File extensions

The list of extensions is mandatory for every language, but does not
necessarily need to exist in every configuration file. It is sufficient
if, for every language, there is a single configuration file that
defines the list of extensions for that language.

### Indentation

The optional field, `indent`, exists to define the indentation method
for that language. Topiary defaults to two spaces `"  "` if it cannot
find the indent field in any configuration file for a specific language.

### Specifying the grammar

Topiary fetches and builds the grammar for you, or a grammar can be
provided by some other method. To have Topiary fetch the grammar for
you, specify the `grammar.source.git` attribute of a language:

```nickel
nickel = {
  extensions = ["ncl"],
  grammar.source.git = {
    git = "https://github.com/nickel-lang/tree-sitter-nickel",
    rev = "43433d8477b24cd13acaac20a66deda49b7e2547",
  },
},
```

To specify a prebuilt grammar, specify the `grammar.source.path`
attribute, which must point to a compiled grammar file on your file
system:

```nickel
nickel = {
  extensions = ["ncl"],
  grammar.source.path = "/path/to/compiled/grammar/file.so",
},
```

> **Note**\
> If you want to link to a grammar file that has already been compiled
> by Topiary itself, those look like `~/.cache/topiary/<LANGUAGE>/<GIT_HASH>.so`
> (or the equivalent for your platform).

For usage in Nix, a `languages_nix.ncl` file is provided that specifies
the paths of every language using the `@nickel@` syntax. These can
easily be replaced with nixpkgs' `substituteAll`.
