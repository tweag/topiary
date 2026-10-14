{
  inputs,
  cargoProfile ? "release",
  runBenchmark ? true,
}:

# reused by root `flake.nix` and debug/dev variant
let
  nixpkgs = inputs.nixpkgs;

  forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;

  topiaryNix = forAllSystems (
    system:
    import ./default.nix {
      pkgs = import nixpkgs { inherit system; };
      inherit (inputs) advisory-db crane rust-overlay;
      inherit cargoProfile runBenchmark;
    }
  );

in
{
  packages = forAllSystems (system: topiaryNix.${system}.packages);
  lib = forAllSystems (system: topiaryNix.${system}.lib);
  checks = forAllSystems (system: topiaryNix.${system}.checks);
  devShells = forAllSystems (system: topiaryNix.${system}.devShells);
}
