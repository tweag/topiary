use clap::ValueEnum;
use topiary::Language;

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

impl From<SupportedLanguage> for Language {
    fn from(language: SupportedLanguage) -> Self {
        match language {
            SupportedLanguage::Json => Language::Json,
            SupportedLanguage::Nickel => Language::Nickel,
            SupportedLanguage::Ocaml => Language::Ocaml,
            SupportedLanguage::OcamlImplementation => Language::OcamlImplementation,
            SupportedLanguage::OcamlInterface => Language::OcamlInterface,
            SupportedLanguage::Toml => Language::Toml,
        }
    }
}
