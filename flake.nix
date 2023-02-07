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

        ## For easy use in https://github.com/cachix/pre-commit-hooks.nix
        lib.pre-commit-hook = {
          enable = true;
          name = "topiary";
          description = "A general code formatter based on tree-sitter.";
          entry = let
            topiary-inplace = pkgs.writeShellApplication {
              name = "topiary-inplace";
              text = ''
                for file; do
                  ${code.app}/bin/topiary --in-place --input-file "$file"
                done
              '';
            };
          in "${topiary-inplace}/bin/topiary-inplace";
          files = "(\\.json$)|(\\.toml$)|(\\.mli?$)";
        };
      }
    );
}

