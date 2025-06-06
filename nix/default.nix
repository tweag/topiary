{
  pkgs ? import <nixpkgs> { },

  # REVIEW: Probably hard to provide without a flake
  advisory-db,
  crane,
  rust-overlay,
}:

let
  overlays = import ./overlays;

  pkgs' = pkgs.appendOverlays [
    overlays.wasm-bindgen-cli
    overlays.callPackageNoOverrides
    rust-overlay.overlays.default
  ];
  inherit (pkgs') callPackageNoOverrides;

  craneLib = crane.mkLib pkgs;

  inherit (callPackageNoOverrides ./packages { inherit advisory-db craneLib; })
    topiaryPkgs
    binPkgs
    ;

  # NOTE: The name could clashes with nixpkgs' lib, which could lead to
  # unexpected behaviours in subsequent `callPackage` statements.
  topiaryLib = callPackageNoOverrides ./lib {
    inherit (topiaryPkgs) topiary-cli;
  };

  checks = callPackageNoOverrides ./checks {
    inherit (pkgs') hello;
    inherit topiaryPkgs;
    inherit (topiaryLib) pre-commit-hook;
  };

  devShells = callPackageNoOverrides ./devShells {
    inherit
      checks
      craneLib
      binPkgs
      topiaryPkgs
      ;
  };

in
{
  # REVIEW: I have kept the separation between “Topiary” vs. “Bin” packages,
  # and I plug it throughout the Nix code, only to flatten it at the very end
  # because that's what flakes want. Maybe we want a flatter organisation from
  # the get-go, where everything gets merged in `packages/default.nix` and we
  # only manipulate the `packages` set?
  packages = topiaryPkgs // binPkgs;
  lib = topiaryLib;
  inherit checks devShells;
}
