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
  writeShellApplication,
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
  inherit (lib) warn getExe;
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

  ## Given a package for Topiary CLI and a configuration, return the CLI with
  ## the configuration hard-coded. It will ignore `.topiary/languages.ncl`,
  ## `$XDG_CONFIG_HOME/topiary/languages.ncl`, and will refuse
  ## `--configuration`. This can be useful to avoid non-reproducibility issues,
  ## or in combination with `prefetchLanguages`.
  makeWithConfiguration =
    package:
    configuration:
    writeShellApplication {
      name = "topiary";
      text = "exec ${getExe package} -C ${toNickelFile "languages.ncl" configuration} \"$@\"";
    } // { inherit configuration; };

  ## Given a package for Topiary CLI (containing its configuration), return an
  ## attrset compatible with https://github.com/cachix/git-hooks.nix.
  makeGitHook =
    package:
    {
      name = "topiary";
      entry = "${getExe package} format";
      files =
        let
          inherit (lib) concatMap attrValues concatStringsSep;
          extensions = concatMap (c: c.extensions) (attrValues package.configuration.languages);
        in
          "\\.(" + concatStringsSep "|" extensions + ")$";
    };

in
{
  inherit
    prefetchLanguages
    prefetchLanguagesFile
    fromNickelFile
    toNickelFile
    makeWithConfiguration
    makeGitHook
    ;
}
