{
  lib,
  writeShellApplication,
  fromNickelFile,
  toNickelFile,
  prefetchLanguages,
  topiary-cli,
  wrapWithConfig,
  defaultConfigPrefetched,
}:

let
  inherit (lib)
    concatMap
    attrValues
    concatStringsSep
    filter
    filterAttrs
    elem
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

  gitHookBinFor =
    inputs@{
      package ? topiary-cli,
      ...
    }:
    wrapWithConfig {
      inherit package;
      config = filterConfig inputs;
    };

  gitHookFor = inputs: {
    enable = false;
    name = "topiary";
    description = "A general code formatter based on tree-sitter.";
    entry = "${gitHookBinFor inputs}/bin/topiary format";
    files =
      let
        extensions = concatMap (c: c.extensions) (attrValues (filterConfig inputs).languages);
      in
      "\\.(" + concatStringsSep "|" extensions + ")$";
  };

  gitHook = gitHookFor { };
  gitHookBin = gitHookBinFor { };

in
{
  inherit
    gitHookFor
    gitHook
    gitHookBinFor
    gitHookBin
    ;
}
