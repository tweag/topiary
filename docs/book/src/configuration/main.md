# Configuration
Topiary is configured using `languages.ncl` files. The `.ncl` extension relates
to [Nickel](https://nickel-lang.org/), a configuration language created by
Tweag. There are up to four sources where Topiary checks for such a file.

### Configuration Sources

At build time the [languages.ncl](https://github.com/tweag/topiary/blob/main/topiary-config/languages.ncl) in the root of
this repository is embedded into Topiary. This file is parsed at
runtime. The purpose of this `languages.ncl` file is to provide sane
defaults for users of Topiary (both the library and the binary).

The next two are read by the Topiary binary at runtime and allow the user to
configure Topiary to their needs. The first is intended to be user specific, and
can thus be found in the configuration directory of the OS:

| OS      | Typical Configuration Path                                        |
| :------ | :---------------------------------------------------------------- |
| Unix    | `/home/alice/.config/topiary/languages.ncl`                      |
| Windows | `C:\Users\Alice\AppData\Roaming\Topiary\config\languages.ncl`    |
| macOS   | `/Users/Alice/Library/Application Support/Topiary/languages.ncl` |

This file is not automatically created by Topiary.

The next source is intended to be a project-specific settings file for
Topiary. When running Topiary in some directory, it will ascend the file
tree until it finds a `.topiary` directory. It will then read any `languages.ncl`
file present in that directory.

Finally, an explicit configuration file may be specified using the
`-C`/`--configuration` command line argument (or the
`TOPIARY_CONFIG_FILE` environment variable). This is intended for
driving Topiary under very specific use-cases.

The Topiary binary parses these sources in the following order.

1. The builtin configuration file.
2. The user configuration file in the OS's configuration directory.
3. The project specific Topiary configuration.
4. The explicit configuration file specified as a CLI argument.

### Configuration Options

The configuration file contains a record of languages. For instance, the one for
Nickel is defined as such:

```nickel
nickel = {
  extensions = ["ncl"],
},
```

The `name` field is used by Topiary to associate the language entry with the
query file and Tree-sitter grammar. This value should be written in lowercase.

The list of extensions is mandatory for every language, but does not necessarily
need to exist in every configuration file. It is sufficient if, for every
language, there is a single configuration file that defines the list of
extensions for that language.

A final optional field, called `indent`, exists to define the indentation method
for that language. Topiary defaults to two spaces `"  "` if it cannot find the
indent field in any configuration file for a specific language.

### Overriding
If one of the sources listed above attempts to define a language configuration
already present in the builtin configuration, Topiary will display a Nickel error.

To understand why, one can read the [Nickel documentation on Merging](https://nickel-lang.org/user-manual/merging).
The short answer is that a priority must be defined. The builtin configuration
has everything defined with priority 0. Any priority above that will replace
any other priority. For example, to override the entire Bash configuration, use the following
Nickel file.

```nickel
{
  languages = {
    bash | priority 1 = {
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
      indent | priority 1 = "    ",
    },
  },
}
```
