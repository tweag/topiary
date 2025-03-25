## Because of dynamic loading, Topiary plays poorly in the Nix sandbox. This
## file introduces two utilities, `prefetchLanguages` and
## `prefetchLanguagesFile` that transform a Topiary configuration into another
## one where all the grammars have been pre-fetched and pre-compiled in Nix
## derivations.

{
  lib,
  fetchgit,
  nickel,
  runCommandNoCC,
  writeText,
  tree-sitter,
}:

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
  inherit (lib) warn;
  inherit (lib.strings) removeSuffix;
  inherit (lib.attrsets) updateManyAttrsByPath;

  prefetchLanguageSourceGit =
    name: source:
    tree-sitter.buildGrammar {
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

  updateByPath = path: update: updateManyAttrsByPath [ { inherit path update; } ];

  ## Given a Topiary configuration as a Nix value, returns the same
  ## configuration, except all language sources have been replaced by a
  ## prefetched and precompiled one. This requires the presence of a `nixHash`
  ## for all sources.
  prefetchLanguages = updateByPath [ "languages" ] (
    mapAttrs (name: updateByPath [ "grammar" "source" ] (prefetchLanguageSource name))
  );

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
    fromNickelFile
    toNickelFile
    ;
}
