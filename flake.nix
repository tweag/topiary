{
  description = "A general code formatter based on tree-sitter.";

  nixConfig = {
    extra-substituters = [
      "https://tweag-topiary.cachix.org"
    ];
    extra-trusted-public-keys = [
      "tweag-topiary.cachix.org-1:8TKqya43LAfj4qNHnljLpuBnxAY/YwEBfzo3kzXxNY0="
    ];
  };

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    rust-overlay.url = "github:oxalica/rust-overlay";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
  };

  outputs =
    inputs:
    let
      nixpkgs = inputs.nixpkgs;
      forAllSystems = nixpkgs.lib.genAttrs nixpkgs.lib.systems.flakeExposed;

      # Default outputs: `release` Cargo profile, benchmark included.
      release = import ./nix/flake-outputs.nix { inherit inputs; };

      # dev/debug Cargo profile, benchmark skipped
      debug = import ./nix/flake-outputs.nix {
        inherit inputs;
        cargoProfile = "dev";
        runBenchmark = false;
      };
    in
    release
    // {
      packages = forAllSystems (
        system:
        release.packages.${system}
        // {
          debug-checks = nixpkgs.legacyPackages.${system}.linkFarmFromDrvs "topiary-ci-checks" (
            builtins.attrValues debug.checks.${system}
          );

          debug-client-app = debug.packages.${system}.client-app;
        }
      );

      devShells = forAllSystems (
        system:
        release.devShells.${system}
        // {
          debug-shell = debug.devShells.${system}.default;
        }
      );
    };
}
