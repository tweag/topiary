{
  callPackageNoOverrides,
  advisory-db,
  craneLib,
}:

let
  topiaryPkgs = callPackageNoOverrides ./topiary.nix {
    inherit advisory-db craneLib;
  };

  binPkgs = callPackageNoOverrides ./bin.nix { };
in

{
  inherit topiaryPkgs binPkgs;
}
