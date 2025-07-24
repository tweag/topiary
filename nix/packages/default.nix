{
  callPackageNoOverrides,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
}:

let
  topiaryPkgs = callPackageNoOverrides ./topiary.nix {
    inherit advisory-db craneLib prefetchLanguagesFile;
  };

  binPkgs = callPackageNoOverrides ./bin.nix { };
in

{
  inherit topiaryPkgs binPkgs;
}
