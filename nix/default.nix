{
  pkgs ? import <nixpkgs> { },
}:

let
  inherit (pkgs.callPackage ./nickelUtils.nix { })
    fromNickelFile
    toNickelFile
    ;

  inherit (pkgs.callPackage ./prefetchLanguages.nix { inherit fromNickelFile toNickelFile; })
    prefetchLanguages
    prefetchLanguagesFile
    ;

  inherit (pkgs.callPackage ./config.nix { inherit fromNickelFile toNickelFile prefetchLanguages; })
    defaultConfig
    defaultConfigFile
    defaultConfigPrefetched
    defaultConfigPrefetchedFile
    wrapWithConfig
    wrapWithConfigFile
    ;

  inherit (pkgs.callPackage ./gitHook.nix { inherit defaultConfigPrefetched wrapWithConfig; })
    gitHook
    gitHookBin
    ;

in

{
  lib = {
    inherit
      fromNickelFile
      toNickelFile
      prefetchLanguages
      prefetchLanguagesFile
      defaultConfig
      defaultConfigFile
      defaultConfigPrefetched
      defaultConfigPrefetchedFile
      wrapWithConfig
      wrapWithConfigFile
      gitHook
      gitHookBin
      ;
  };
}
