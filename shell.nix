# Allows `nix-shell` without having to go trough the trouble of pinning the same
# version as is done by the flake.
{ pkgs ? import <nixpkgs> { }
, checks ? { }
, craneLib
}:
let
  update-wasm-grammars = pkgs.writeShellApplication {
    name = "update-wasm-grammars";

    runtimeInputs = with pkgs; [
      emscripten
      git
      toml2json
      tree-sitter
    ];

    text = builtins.readFile ./update-wasm-grammars.sh;
  };
in
craneLib.devShell {
  inherit checks;

  packages = with pkgs; [
    update-wasm-grammars
    cargo-flamegraph
    rust-analyzer
  ];
}
