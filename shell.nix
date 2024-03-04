# Allows `nix-shell` without having to go trough the trouble of pinning the same
# version as is done by the flake.
{ pkgs
, checks ? { }
, craneLib
, binPkgs
}:
craneLib.devShell {
  inherit checks;

  packages = with pkgs; with binPkgs; [
    cargo-flamegraph
    rust-analyzer

    # Our own scripts
    generate-coverage
    playground
    update-wasm-app
    update-wasm-grammars
    verify-documented-usage
  ];
}
