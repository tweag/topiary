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
        system: let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [rust-overlay.overlays.default];
          };
          code = pkgs.callPackage ./default.nix {inherit advisory-db crane rust-overlay nix-filter;};
        in {
          packages = with code; {
            inherit topiary-playground;
            default = topiary-cli;
          };

          checks = {
            inherit (code) clippy clippy-wasm fmt topiary-lib topiary-cli topiary-playground audit benchmark;

            ## Check that the `lib.pre-commit-hook` output builds/evaluates
            ## correctly. `deepSeq e1 e2` evaluates `e1` strictly in depth before
            ## returning `e2`. We use this trick because checks need to be
            ## derivations, which `lib.pre-commit-hook` is not.
            pre-commit-hook = builtins.deepSeq self.lib.${system}.pre-commit-hook pkgs.hello;
          };

          ## For easy use in https://github.com/cachix/pre-commit-hooks.nix
          lib.pre-commit-hook = {
            enable = true;
            name = "topiary";
            description = "A general code formatter based on tree-sitter.";
            entry = let
              topiary-inplace = pkgs.writeShellApplication {
                name = "topiary-inplace";
                text = ''
                  ${code.topiary-cli}/bin/topiary --in-place --input-files "$@";
                '';
              };
            in "${topiary-inplace}/bin/topiary-inplace";
            types = ["text"];
          };
        }
      );
}
