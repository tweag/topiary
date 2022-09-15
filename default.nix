{ pkgs, crane, nix-filter }:
let
  craneLib = crane.mkLib pkgs;
in
{
  app = craneLib.buildPackage {
    src = nix-filter.lib.filter {  
      root = ./.;
      include = [
        "benches"
        "Cargo.lock"
        "Cargo.toml"
        "languages"
        "src"
        "tests"
      ];
    };    
    nativeBuildInputs = [ pkgs.libiconv ];
    cargoTestCommand = "cargo test --profile release && cargo bench --profile release";
  };
}