# Allows `nix-shell` without having to go trough the trouble of pinning the same
# version as is done by the flake.
{ pkgs
, checks ? { }
, craneLib
, binPkgs
, optionals ? true
}:
craneLib.devShell
  (
    {
      inherit checks;
    }
      //
    (if optionals then {
      packages = with pkgs; with binPkgs; [
        cargo-dist
        cargo-flamegraph
        # https://crane.dev/API.html?highlight=mold#cranelibcargonextest
        # cargo-nextest
        rust-analyzer
        # # Look into mold-wrapped or `pkgs.stdenvAdapters.useMoldLinker`
        # # https://crane.dev/API.html?highlight=mold#cranelibdevshell
        # mold-wrapped

        pkg-config
        openssl.dev

        # Our own scripts
        # FIXME: Broken
        # generate-coverage
        update-wasm-app
        update-wasm-grammars
        verify-documented-usage
      ]
      ++ pkgs.lib.optionals (!stdenv.isDarwin) [
        playground
      ];
    } else { })
  )
