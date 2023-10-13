//! Configuration collation functionality

use clap::ValueEnum;

/// Collation mode for configuration values
// NOTE The enum variants are in "natural" order, rather than
// sorted lexicographically, for the sake of the help text
#[derive(Clone, Debug, ValueEnum)]
pub enum CollationMode {
    /// When multiple sources of configuration are available, matching items are updated from the
    /// higher priority source, with collections merged as the union of sets.
    Merge,

    /// When multiple sources of configuration are available, matching items (including
    /// collections) are superseded from the higher priority source.
    Revise,

    /// When multiple sources of configuration are available, the highest priority source is taken.
    /// All values from lower priority sources are discarded.
    Override,
}

impl CollationMode {
    pub fn collate_toml(&self, base: toml::Value, graft: toml::Value) -> toml::Value {
        // Map collation modes to merge depths for the TOML collation (see `collate_toml`)
        let merge_depth = match self {
            CollationMode::Merge => 4,
            CollationMode::Revise => 2,
            CollationMode::Override => return graft,
        };

        collate_toml(base, graft, merge_depth)
    }
}

/// Collate two TOML documents, merging values from `graft` onto `base`.
///
/// Arrays of tables with a `name` key (e.g., our `[[language]]` tables) are always merged; that
/// is, the union of the `base` and `graft` is taken. Otherwise, the `merge_depth` controls the
/// collation of arrays, resulting in concatenation. This can leave duplicates, in the collated
/// TOML, but for Topiary, this only matters for our `Languages::extensions`, which is implemented
/// as a `HashSet`; thus deserialisation will deduplicate for us.
///
/// When a table exists in both `base` and `graft`, the merged table consists of all keys in
/// `base`'s table unioned with all keys in `graft` with the values of `graft` being merged
/// recursively onto values of `base`.
///
/// NOTE This collation function is forked from Helix, licensed under MPL-2.0
/// * Repo: https://github.com/helix-editor/helix
/// * Rev:  df09490
/// * Path: helix-loader/src/lib.rs
fn collate_toml(base: toml::Value, graft: toml::Value, merge_depth: usize) -> toml::Value {
    use toml::Value;

    fn get_name(v: &Value) -> Option<&str> {
        v.get("name").and_then(Value::as_str)
    }

    match (base, graft, merge_depth) {
        // Fallback to the graft value if the recursion depth bottoms out
        (_, graft, 0) => graft,

        (Value::Array(mut base_items), Value::Array(graft_items), _) => {
            for rvalue in graft_items {
                // If our graft value has a `name` key, then we're dealing with a `[[language]]`
                // table. In which case, pop it -- if it exists -- from the base array.
                let language = get_name(&rvalue)
                    .and_then(|rname| base_items.iter().position(|v| get_name(v) == Some(rname)))
                    .map(|lpos| base_items.remove(lpos));

                let mvalue = match language {
                    // Merge matching language tables
                    Some(lvalue) => collate_toml(lvalue, rvalue, merge_depth - 1),

                    // Collate everything else
                    None => rvalue,
                };

                base_items.push(mvalue);
            }

            Value::Array(base_items)
        }

        (Value::Table(mut base_map), Value::Table(graft_map), _) => {
            for (rname, rvalue) in graft_map {
                match base_map.remove(&rname) {
                    Some(lvalue) => {
                        let merged_value = collate_toml(lvalue, rvalue, merge_depth - 1);
                        base_map.insert(rname, merged_value);
                    }
                    None => {
                        base_map.insert(rname, rvalue);
                    }
                }
            }

            Value::Table(base_map)
        }

        // Fallback to the graft value for everything else
        (_, graft, _) => graft,
    }
}

#[cfg(test)]
mod test_config_collation {
    use super::CollationMode;
    use crate::configuration::format::Configuration;

    // NOTE PartialEq for toml::Value is (understandably) order sensitive over array elements, so
    // we deserialse to `topiary::Configuration` for equality testing. This also has the effect of
    // side-stepping potential duplication, from concatenation, when using `CollationMode::Merge`.

    static BASE: &str = r#"
        [[language]]
        name = "example"
        extensions = ["eg"]

        [[language]]
        name = "demo"
        extensions = ["demo"]
    "#;

    static GRAFT: &str = r#"
        [[language]]
        name = "example"
        extensions = ["example"]
        indent = "\t"
    "#;

    #[test]
    fn merge() {
        let base = toml::from_str(BASE).unwrap();
        let graft = toml::from_str(GRAFT).unwrap();

        let merged: Configuration = CollationMode::Merge
            .collate_toml(base, graft)
            .try_into()
            .unwrap();

        let expected: Configuration = toml::from_str(
            r#"
            [[language]]
            name = "example"
            extensions = ["eg", "example"]
            indent = "\t"

            [[language]]
            name = "demo"
            extensions = ["demo"]
            "#,
        )
        .unwrap();

        assert_eq!(merged, expected);
    }

    #[test]
    fn revise() {
        let base = toml::from_str(BASE).unwrap();
        let graft = toml::from_str(GRAFT).unwrap();

        let revised: Configuration = CollationMode::Revise
            .collate_toml(base, graft)
            .try_into()
            .unwrap();

        let expected: Configuration = toml::from_str(
            r#"
            [[language]]
            name = "example"
            extensions = ["example"]
            indent = "\t"

            [[language]]
            name = "demo"
            extensions = ["demo"]
            "#,
        )
        .unwrap();

        assert_eq!(revised, expected);
    }
}
