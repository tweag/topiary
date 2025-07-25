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
    inherit (binPkgs) mdbook-generate-nix-documentation;
  };
in

{
  inherit topiaryPkgs binPkgs;
}
