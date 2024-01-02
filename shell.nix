# Allows `nix-shell` without having to go trough the trouble of pinning the same
# version as is done by the flake.
{ pkgs ? import <nixpkgs> { }
, checks ? { }
,
}: pkgs.mkShell {
  inputsFrom = builtins.attrValues checks;

  buildInputs = with pkgs; [
    cargo-flamegraph
    rust-analyzer
  ];
}
