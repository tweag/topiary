{
  callPackageNoOverrides,
  advisory-db,
  craneLib,
  prefetchLanguagesFile,
  prefetchLanguagesNickelFile,
  cargoProfile ? "release",
  runBenchmark ? true,
}:

let
  binPkgs = callPackageNoOverrides ./bin.nix { };

  topiaryPkgs = callPackageNoOverrides ./topiary.nix {
    inherit
      advisory-db
      craneLib
      prefetchLanguagesFile
      prefetchLanguagesNickelFile
      cargoProfile
      runBenchmark
      ;
  };
in

{
  inherit topiaryPkgs binPkgs;
}
