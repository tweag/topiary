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

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        code = pkgs.callPackage ./. { inherit nixpkgs system advisory-db crane rust-overlay nix-filter; };
      in
      {
        packages = with code; {
          inherit wasm-app;
          default = app;
        };
        checks = with code; {
          inherit app clippy fmt audit benchmark;
        };

        ## For easy use in https://github.com/cachix/pre-commit-hooks.nix
        lib.pre-commit-hook = {
          enable = true;
          name = "topiary";
          description = "A general code formatter based on tree-sitter.";
          entry =
            let
              topiary-inplace = pkgs.writeShellApplication {
                name = "topiary-inplace";
                text = ''
                  for FILE; do
                    if ${code.app}/bin/topiary --in-place --input-file "$FILE"; then
                      continue
                    else
                      EXIT=$?
                      if (( EXIT == 6 )); then
                        # Skip over language detection errors
                        continue
                      else
                        exit $EXIT
                      fi
                    fi
                  done
                '';
              };
            in
            "${topiary-inplace}/bin/topiary-inplace";
          types = [ "text" ];
        };
      }
    );
}

