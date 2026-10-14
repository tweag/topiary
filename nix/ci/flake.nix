{
  description = "Topiary CI checks (dev Cargo profile, benchmarks skipped)";

  nixConfig = {
    extra-substituters = [
      "https://tweag-topiary.cachix.org"
    ];
    extra-trusted-public-keys = [
      "tweag-topiary.cachix.org-1:8TKqya43LAfj4qNHnljLpuBnxAY/YwEBfzo3kzXxNY0="
    ];
  };

  inputs = {
    # Allows "following" the root flake reivision
    # `nix flake update ./nix/ci` still needs to be run separately.
    topiary.url = "path:../..";

    nixpkgs.follows = "topiary/nixpkgs";
    crane.follows = "topiary/crane";
    rust-overlay.follows = "topiary/rust-overlay";
    advisory-db.follows = "topiary/advisory-db";
  };

  outputs =
    inputs@{ nixpkgs, ... }:
    let
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;

      base = import ../flake-outputs.nix {
        inherit inputs;
        cargoProfile = "dev";
        runBenchmark = false;
      };
    in
    {
      checks = base.checks;

      devShells = base.devShells;

      # Exposed so that CI can build the `client-app` example in the `dev`
      # profile via `nix build ./nix/ci#client-app`.
      packages = forAllSystems (system: {
        inherit (base.packages.${system}) client-app;
      });
    };
}
