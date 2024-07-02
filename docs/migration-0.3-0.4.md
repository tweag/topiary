# Topiary CLI Migration Guide
Topiary 0.4 has switched away from toml to [Nickel](https://nickel-lang.org/) for its configuration language.
For a help on how to configure Topiary using Nickel see the [`README`](/README.md).
This guide is only there to help you migrate from toml to Nickel.

### Migrating to Nickel
## Before
```toml
[[language]]
name = "bash"
extensions = ["sh", "bash"]

[[language]]
name = "rust"
extensions = ["rs"]
indent = "    " # 4 spaces
```

## After
```nickel
{
  "bash" = {
    extensions = ["sh", "bash"],
  },

  "rust" = {
    extensions = ["rs"],
    indent = "    ", # 4 spaces
  },
}
```

### Keeping TOML
Nickel comes with builtin TOML importing, so if you want to keep using your old languages.toml file, you can use the following nickel expression.
```nickel
let toml = import "./languages.toml" in
# Transforms the array into something we can convert into a record
let array = std.array.map (fun x => { field = x.name, value = std.record.remove "name" x }) toml.language in
# Actually convert to a record
let record = std.record.from_array array in
record
```
