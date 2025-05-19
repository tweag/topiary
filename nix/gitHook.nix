## Pre-commit Git hook compatible with https://github.com/cachix/git-hooks.nix.
## If the hook is to be used in a sandboxed environment, for instance as part of
## `nix flake check`, then one should provide a prefetched configuration.

{
  lib,
  defaultConfigPrefetched,
  wrapWithConfig,
}:

let
  inherit (builtins)
    concatStringsSep
    elem
    filter
    ;
  inherit (lib)
    filterAttrs
    ;

  filterConfig =
    {
      config ? defaultConfigPrefetched,
      includeLanguages ? null, # `null` means all languages
      excludeLanguages ? [ ],
      ...
    }:
    let
      unsupportedLanguages = filter (lang: !(config.languages ? ${lang})) (
        (if includeLanguages == null then [ ] else includeLanguages) ++ excludeLanguages
      );
    in
    if includeLanguages != null && excludeLanguages != [ ] then
      throw "gitHook: cannot pass both `includeLanguages` and `excludeLanguages`."
    else if unsupportedLanguages != [ ] then
      throw "gitHook: unsupported languages: ${concatStringsSep ", " unsupportedLanguages}."
    else if includeLanguages != null then
      config
      // {
        languages = filterAttrs (lang: _: elem lang includeLanguages) config.languages;
      }
    else
      config
      // {
        languages = filterAttrs (lang: _: !(elem lang excludeLanguages)) config.languages;
      };

  ## In case a client of our Nix code wants to get a Git hook but also acquire
  ## the same Topiary, for instance to expose it in their environment.
  ##
  ## REVIEW: I would like expose this repository's `topiary-cli` package as the
  ## default `bin` argument, but this would lead to a nasty recursion between
  ## `/default.nix` and `/nix`.
  gitHookBin = inputs@{ bin, ... }: wrapWithConfig bin (filterConfig inputs);

  gitHook = inputs: {
    enable = false;

    name = "topiary";
    entry = "${gitHookBin inputs}/bin/topiary format";

    files =
      let
        inherit (lib) concatMap attrValues concatStringsSep;
        extensions = concatMap (c: c.extensions) (attrValues (filterConfig inputs).languages);
      in
      "\\.(" + concatStringsSep "|" extensions + ")$";
  };

in
{
  inherit
    gitHook
    gitHookBin
    ;
}
