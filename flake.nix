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

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    rust-overlay.url = "github:oxalica/rust-overlay";

    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };

    tree-sitter-nickel = {
      url = "github:nickel-lang/tree-sitter-nickel";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, ... }@inputs:
    let
      supportedSystems = nixpkgs.lib.systems.flakeExposed;

      pkgsFor = nixpkgs.lib.genAttrs supportedSystems (system: rec {
        pkgs = import nixpkgs {
          inherit system;
          overlays = [
            self.overlays.tree-sitter-grammars
            self.overlays.wasm-bindgen-cli
            inputs.rust-overlay.overlays.default
          ];
        };

        topiaryPkgs = pkgs.callPackage ./default.nix {
          inherit (inputs) advisory-db crane rust-overlay;
          inherit (pkgs.tree-sitter-grammars) tree-sitter-nickel;
          craneLib = inputs.crane.mkLib pkgs;
        };

        binPkgs = pkgs.callPackage ./bin/default.nix { };
      });

      forAllSystems = fn: nixpkgs.lib.genAttrs supportedSystems (system: fn rec {
        inherit system;
        inherit (pkgsFor.${system}) pkgs topiaryPkgs binPkgs;
        inherit (pkgs) lib;
        craneLib = inputs.crane.mkLib pkgs;
      });
    in
    {
      overlays = {
        tree-sitter-grammars = final: prev: {
          # Nickel *should* have an overlay like this already
          tree-sitter-grammars = prev.tree-sitter-grammars // {
            tree-sitter-nickel = inputs.tree-sitter-nickel.packages.${prev.system}.default;
          };
        };

        wasm-bindgen-cli = final: prev:
          let
            cargoLock = builtins.fromTOML (builtins.readFile ./Cargo.lock);
            wasmBindgenCargoVersions = builtins.map ({ version, ... }: version) (builtins.filter ({ name, ... }: name == "wasm-bindgen") cargoLock.package);
            wasmBindgenVersion = assert builtins.length wasmBindgenCargoVersions == 1; builtins.elemAt wasmBindgenCargoVersions 0;
          in
          {
            wasm-bindgen-cli = prev.wasm-bindgen-cli.override {
              version = wasmBindgenVersion;
              hash = "sha256-f/RK6s12ItqKJWJlA2WtOXtwX4Y0qa8bq/JHlLTAS3c=";
              cargoHash = "sha256-3vxVI0BhNz/9m59b+P2YEIrwGwlp7K3pyPKt4VqQuHE=";
            };
          };
      };

      packages = forAllSystems ({ system, pkgs, topiaryPkgs, binPkgs, ... }: {
        inherit (topiaryPkgs)
          topiary-playground
          topiary-queries
          client-app;

        topiary-cli = topiaryPkgs.topiary-cli { };
        topiary-cli-nix = topiaryPkgs.topiary-cli { nixSupport = true; };

        inherit (binPkgs)
          # FIXME: Broken
          # generate-coverage
          playground
          update-wasm-app
          update-wasm-grammars
          verify-documented-usage;

        default = self.packages.${system}.topiary-cli;
      });

      checks = forAllSystems ({ system, pkgs, topiaryPkgs, ... }: {
        # NOTE: The following checks have been removed as WASM
        # and playground development has moved to the playground branch:
        # - clippy-wasm
        # - topiary-playground
        inherit (topiaryPkgs) clippy fmt topiary-core audit benchmark;
        topiary-cli = self.packages.${system}.topiary-cli;

        ## Check that the `lib.pre-commit-hook` output builds/evaluates
        ## correctly. `deepSeq e1 e2` evaluates `e1` strictly in depth before
        ## returning `e2`. We use this trick because checks need to be
        ## derivations, which `lib.pre-commit-hook` is not.
        pre-commit-hook = builtins.deepSeq self.lib.${system}.pre-commit-hook pkgs.hello;
      });

      devShells = forAllSystems ({ system, pkgs, craneLib, topiaryPkgs, binPkgs, ... }:
        {
          default = pkgs.callPackage ./shell.nix { checks = self.checks.${system}; inherit craneLib; inherit binPkgs; };
          light = pkgs.callPackage ./shell.nix {
            checks = /* checksLight */ {
              inherit (topiaryPkgs) clippy fmt topiary-core;
              topiary-cli = self.packages.${system}.topiary-cli;
            };
            inherit craneLib;
            inherit binPkgs;
            optionals = false;
          };
          wasm = pkgs.callPackage ./shell.nix { checks = self.checks.${system}; craneLib = topiaryPkgs.passthru.craneLibWasm; inherit binPkgs; };
        });

      ## For easy use in https://github.com/cachix/pre-commit-hooks.nix
      lib = forAllSystems ({ system, lib, ... }: {
        pre-commit-hook = {
          enable = true;
          name = "topiary";
          description = "A general code formatter based on tree-sitter.";
          entry = "${lib.getExe self.packages.${system}.topiary-cli} fmt";
          types = [ "text" ];
        };
      });
    };
}
