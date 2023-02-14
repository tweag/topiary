use clap::ArgEnum;
use topiary::Language;

#[derive(ArgEnum, Clone, Copy, Debug)]
pub enum SupportedLanguage {
    Json,
    Toml,
    Ocaml,
    OcamlImplementation,
    OcamlInterface,
    // Any other entries in crate::Language are experimental and won't be
    // exposed in the CLI. They can be accessed using --query language/foo.scm
    // instead.
}

impl From<SupportedLanguage> for Language {
    fn from(language: SupportedLanguage) -> Self {
        match language {
            SupportedLanguage::Json => Language::Json,
            SupportedLanguage::Toml => Language::Toml,
            SupportedLanguage::Ocaml => Language::Ocaml,
            SupportedLanguage::OcamlImplementation => Language::OcamlImplementation,
            SupportedLanguage::OcamlInterface => Language::OcamlInterface,
        }
    }
}
