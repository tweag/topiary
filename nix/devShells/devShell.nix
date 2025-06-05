{
  pkgs,
  checks ? { },
  craneLib,
  binPkgs,
  optionals ? true,
}:

craneLib.devShell (
  {
    inherit checks;
  }
  // (
    if optionals then
      {
        packages =
          with pkgs;
          with binPkgs;
          [
            cargo-dist
            cargo-flamegraph
            rust-analyzer
            mdbook

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
      }
    else
      { }
  )
)
