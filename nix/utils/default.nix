{
  callPackageNoOverrides,
}:

let
  inherit (callPackageNoOverrides ./nickelUtils.nix { })
    toNickelFile
    fromNickelFile
    ;

  inherit
    (callPackageNoOverrides ./prefetchLanguages.nix {
      inherit toNickelFile fromNickelFile;
    })
    prefetchLanguages
    prefetchLanguagesFile
    ;
in

{
  inherit
    toNickelFile
    fromNickelFile
    prefetchLanguages
    prefetchLanguagesFile
    ;
}
