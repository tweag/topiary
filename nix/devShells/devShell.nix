{
  pkgs,
  checks ? { },
  craneLib,
  binPkgs,
  topiaryPkgs ? { },
  includeExtraPackages ? true,
}:

let
  # mold linker not supported on darwin.
  # https://github.com/ipetkov/crane/blob/master/docs/API.md#cranelibdevshell
  devShell =
    if pkgs.stdenv.hostPlatform.isLinux then
      craneLib.devShell.override {
        mkShell = pkgs.mkShell.override {
          stdenv = pkgs.stdenvAdapters.useMoldLinker pkgs.stdenv;
        };
      }
    else
      craneLib.devShell;
in
devShell (
  {
    inherit checks;
  }
  // (
    if includeExtraPackages then
      {
        packages =
          with pkgs;
          with binPkgs;
          [
            cargo-dist
            cargo-flamegraph
            rust-analyzer

            jq
            nixdoc

            topiaryPkgs.mdbook
            mdbook-man
            topiaryPkgs.mdbook-manmunge

            pkg-config
            openssl.dev

            # Our own scripts
            # FIXME: Broken
            # generate-coverage
            generate-nix-documentation
            verify-documented-usage
          ]
          ++ pkgs.lib.optionals (!stdenv.hostPlatform.isDarwin) [
            playground
          ];
      }
    else
      { }
  )
)
