{
  description = "A general code formatter based on tree-sitter.";

  inputs = {
    crane.url = "github:ipetkov/crane";
    flake-utils.follows = "crane/flake-utils";
    nixpkgs.follows = "crane/nixpkgs";
  };

  outputs = inputs: with inputs;
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        code = pkgs.callPackage ./. { inherit crane; };
      in {
        packages.default = code.app;
      }
    );
}