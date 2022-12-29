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
    advisory-db = {
      url = "github:rustsec/advisory-db";
      flake = false;
    };
    crane.url = "github:ipetkov/crane";
    flake-utils.follows = "crane/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
    nixpkgs.follows = "crane/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        code = pkgs.callPackage ./. { inherit advisory-db crane nix-filter; };
      in {
        packages.default = code.app;
        checks = with code; {
          inherit app clippy fmt audit benchmark;
        };
        devShells.default = pkgs.mkShell {
          inputsFrom = [ code.app ];
          packages = with pkgs; [
            cargo
            rust-analyzer
            rustc
            rustfmt
          ];
        };
      }
    );
}

