# Allows `nix-shell` without having to go trough the trouble of pinning the same
# version as is done by the flake.
{
  pkgs ? import <nixpkgs> {},
  checks ? {},
}: let
  update-wasm-grammars = pkgs.writeShellApplication {
    name = "update-wasm-grammars";

    runtimeInputs = with pkgs; [git tree-sitter emscripten];

    text = builtins.readFile ./update-wasm-grammars.sh;
  };
in
  pkgs.mkShell {
    inputsFrom = builtins.attrValues checks;

    buildInputs = with pkgs; [
      update-wasm-grammars
      cargo-flamegraph
      rust-analyzer
    ];
  }
