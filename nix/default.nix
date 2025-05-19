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
      ;
  };
}
