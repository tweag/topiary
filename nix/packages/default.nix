{
  callPackageNoOverrides,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
}:

let
  binPkgs = callPackageNoOverrides ./bin.nix { };

  topiaryPkgs = callPackageNoOverrides ./topiary.nix {
    inherit advisory-db craneLib prefetchLanguagesFile;
  };
in

{
  inherit topiaryPkgs binPkgs;
}
