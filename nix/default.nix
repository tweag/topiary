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

in

{
  lib = {
    inherit
      fromNickelFile
      toNickelFile
      prefetchLanguages
      prefetchLanguagesFile
      ;
  };
}
