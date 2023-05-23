use clap::ValueEnum;
use topiary::{Configuration, Language};

#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum SupportedLanguage {
    Json,
    Nickel,
    Ocaml,
    OcamlImplementation,
    OcamlInterface,
    Toml,
    // Any other entries in crate::Language are experimental and won't be
    // exposed in the CLI. They can be accessed using --query language/foo.scm
    // instead.
}

impl SupportedLanguage {
    pub fn to_language(self, configuration: &Configuration) -> &Language {
        let name = match self {
            SupportedLanguage::Json => "json",
            SupportedLanguage::Nickel => "nickel",
            SupportedLanguage::Ocaml | SupportedLanguage::OcamlImplementation => "ocaml",
            SupportedLanguage::OcamlInterface => "ocaml_interface",
            SupportedLanguage::Toml => "toml",
        };
        for lang in &configuration.language {
            if lang.name == name {
                return lang;
            }
        }
        // Every supported language MUST have an entry in the builtin
        // configuration, and so there should always be a match.
        unreachable!()
    }
}
