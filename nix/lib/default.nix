{
  callPackageNoOverrides,
  topiaryUtils,
  topiary-cli,
}:

let
  inherit
    (callPackageNoOverrides ./config.nix {
      inherit (topiaryUtils) fromNickelFile toNickelFile prefetchLanguages;
    })
    defaultConfig
    defaultConfigFile
    defaultConfigPrefetched
    defaultConfigPrefetchedFile
    wrapWithConfig
    wrapWithConfigFile
    ;

  inherit
    (callPackageNoOverrides ./gitHook.nix {
      inherit topiary-cli defaultConfigPrefetched wrapWithConfig;
      inherit (topiaryUtils) prefetchLanguages toNickelFile fromNickelFile;
    })
    gitHookFor
    gitHook
    gitHookBinFor
    gitHookBin
    ;
in

{
  inherit
    defaultConfig
    defaultConfigFile
    defaultConfigPrefetched
    defaultConfigPrefetchedFile
    wrapWithConfig
    wrapWithConfigFile
    gitHookFor
    gitHook
    gitHookBinFor
    gitHookBin
    ;
}
