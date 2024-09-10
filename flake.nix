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

    flake-utils.url = "github:numtide/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
  };

  outputs = inputs:
    with inputs;
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        wasm-bindgen-cli-overlay = final: prev:
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

        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default wasm-bindgen-cli-overlay ];
        };

        craneLib = crane.mkLib pkgs;

        topiaryPkgs = pkgs.callPackage ./default.nix { inherit advisory-db crane rust-overlay nix-filter craneLib; };
        binPkgs = pkgs.callPackage ./bin/default.nix { };
      in
      {
        packages = {
          inherit (topiaryPkgs)
            topiary-cli
            topiary-playground
            topiary-queries
            client-app;

          inherit (binPkgs)
            # FIXME: Broken
            # generate-coverage
            playground
            update-wasm-app
            update-wasm-grammars
            verify-documented-usage;

          default = topiaryPkgs.topiary-cli;
        };

        checks = {
          inherit (topiaryPkgs) clippy clippy-wasm fmt topiary-core topiary-cli topiary-playground audit benchmark;

          ## Check that the `lib.pre-commit-hook` output builds/evaluates
          ## correctly. `deepSeq e1 e2` evaluates `e1` strictly in depth before
          ## returning `e2`. We use this trick because checks need to be
          ## derivations, which `lib.pre-commit-hook` is not.
          pre-commit-hook = builtins.deepSeq self.lib.${system}.pre-commit-hook pkgs.hello;
        };

        devShells =
          let
            checksLight = {
              inherit (topiaryPkgs) clippy fmt topiary-core topiary-cli;
            };
          in
          {
            default = pkgs.callPackage ./shell.nix { checks = self.checks.${system}; inherit craneLib; inherit binPkgs; };
            light = pkgs.callPackage ./shell.nix { checks = checksLight; inherit craneLib; inherit binPkgs; optionals = false; };
            wasm = pkgs.callPackage ./shell.nix { checks = self.checks.${system}; craneLib = topiaryPkgs.passtru.craneLibWasm; inherit binPkgs; };
          };

        ## For easy use in https://github.com/cachix/pre-commit-hooks.nix
        lib.pre-commit-hook = {
          enable = true;
          name = "topiary";
          description = "A general code formatter based on tree-sitter.";
          entry = "${topiaryPkgs.topiary-cli}/bin/topiary fmt";
          types = [ "text" ];
        };
      }
    );
}
