{ pkgs, ... }:

let
  inherit (builtins)
    attrNames
    concatStringsSep
    mapAttrs
    toFile
    readFile
    toJSON
    fromJSON
    baseNameOf
    ;
  inherit (pkgs)
    fetchgit
    nickel
    runCommandNoCC
    writeText
    ;
  inherit (pkgs.lib) warn;
  inherit (pkgs.lib.strings) removeSuffix;
  inherit (pkgs.tree-sitter) buildGrammar;

  prefetchLanguageSourceGit =
    name: source:
    buildGrammar {
      language = name;
      version = source.rev;
      src = fetchgit {
        url = source.git;
        rev = source.rev;
        hash =
          if source ? "nixHash" then
            source.nixHash
          else
            warn "Language `${name}`: no nixHash provided - using dummy value" "sha256-AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=";
      };
      location = if source ? "subdir" then source.subdir else null;
    };

  prefetchLanguageSource =
    name: source:
    if source ? "path" then
      { inherit (source) path; }
    else if source ? "git" then
      { path = "${prefetchLanguageSourceGit name source.git}/parser"; }
    else
      throw ("Unsupported Topiary language sources: " ++ concatStringsSep ", " (attrNames source));

  ## Given a Topiary configuration as a Nix value, returns the same
  ## configuration, except all language sources have been replaced by a
  ## prefetched and precompiled one. This requires the presence of a `nixHash`
  ## for all sources.
  prefetchLanguages =
    topiaryConfig:
    topiaryConfig
    // {
      languages = mapAttrs (
        name: languageConfig:
        languageConfig
        // {
          grammar = languageConfig.grammar // {
            source = prefetchLanguageSource name languageConfig.grammar.source;
          };
        }
      ) topiaryConfig.languages;
    };

  toNickelFile =
    name: e:
    let
      jsonFile = writeText "${removeSuffix ".ncl" name}.json" (toJSON e);
    in
    writeText name "import \"${jsonFile}\"";

  fromNickelFile =
    path:
    let
      jsonDrv = runCommandNoCC "${removeSuffix ".ncl" (baseNameOf path)}.json" { } ''
        ${nickel}/bin/nickel export ${path} > $out
      '';
    in
    fromJSON (readFile "${jsonDrv}");

  ## Same as `prefetchLanguages`, but expects a path to a Nickel file, and
  ## produces a path to another Nickel file.
  prefetchLanguagesFile =
    topiaryConfigFile:
    toNickelFile "${removeSuffix ".ncl" (baseNameOf topiaryConfigFile)}-prefetched.ncl" (
      prefetchLanguages (fromNickelFile topiaryConfigFile)
    );

in
{
  inherit
    prefetchLanguages
    prefetchLanguagesFile
    ;
}
