{
  callPackageNoOverrides,
}:

let
  inherit (callPackageNoOverrides ./nickelUtils.nix { })
    toJSONFile
    fromNickelFile
    ;

  inherit
    (callPackageNoOverrides ./prefetchLanguages.nix {
      inherit toJSONFile fromNickelFile;
    })
    prefetchLanguages
    prefetchLanguagesFile
    ;
in

{
  inherit
    toJSONFile
    fromNickelFile
    prefetchLanguages
    prefetchLanguagesFile
    ;
}
