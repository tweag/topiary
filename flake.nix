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

    tree-sitter-nickel = {
      url = "github:nickel-lang/tree-sitter-nickel";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    tree-sitter-openscad = {
      url = "github:mkatychev/tree-sitter-openscad";
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
        topiaryArgs = {
          inherit (inputs) advisory-db crane rust-overlay;
          inherit (pkgs.tree-sitter-grammars) tree-sitter-nickel tree-sitter-openscad;
          craneLib = inputs.crane.mkLib pkgs;
        };

        topiaryPkgs = pkgs.callPackage ./default.nix topiaryArgs;
        topiaryPkgsDev = pkgs.callPackage ./default.nix (topiaryArgs // { release = false; });

        binPkgs = pkgs.callPackage ./bin/default.nix { };
      });

      forAllSystems = fn: nixpkgs.lib.genAttrs supportedSystems (system: fn rec {
        inherit system;
        inherit (pkgsFor.${system}) pkgs topiaryPkgs topiaryPkgsDev binPkgs;
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
            tree-sitter-openscad = inputs.tree-sitter-openscad.packages.${prev.system}.default;
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
              hash = "sha256-3RJzK7mkYFrs7C/WkhW9Rr4LdP5ofb2FdYGz1P7Uxog=";
              cargoHash = "sha256-tD0OY2PounRqsRiFh8Js5nyknQ809ZcHMvCOLrvYHRE=";
            };
          };
      };

      packages = forAllSystems ({ system, pkgs, topiaryPkgs, topiaryPkgsDev, binPkgs, ... }: {
        inherit (topiaryPkgs)
          topiary-playground
          topiary-queries
          benchmark
          client-app;

        client-app-dev = topiaryPkgsDev.client-app;
        topiary-cli =  topiaryPkgs.topiary-cli { };
        topiary-cli-dev = topiaryPkgsDev.topiary-cli { };
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

      checks = forAllSystems ({ system, pkgs, topiaryPkgsDev, ... }: {
        # NOTE: The following checks have been removed as WASM
        # and playground development has moved to the playground branch:
        # - clippy-wasm
        # - topiary-playground
        inherit (topiaryPkgsDev) clippy fmt topiary-core audit;
        topiary-cli = self.packages.${system}.topiary-cli-dev;

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
