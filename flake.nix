{
  description = "A general code formatter based on tree-sitter.";

  nixConfig = {
    extra-substituters = [
      "https://tweag-tree-sitter-formatter.cachix.org"
    ];
    extra-trusted-public-keys = [
      "tweag-tree-sitter-formatter.cachix.org-1:R95oCa9JV/Cu8dtdFZY55HLFqJ3ASh34dXh7o7LeL5Y="
    ];
  };
  
  inputs = {
    crane.url = "github:ipetkov/crane";
    flake-utils.follows = "crane/flake-utils";
    nix-filter.url = "github:numtide/nix-filter";
    nixpkgs.follows = "crane/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        code = pkgs.callPackage ./. { inherit crane nix-filter; };
      in {
        packages.default = code.app;
      }
    );
}