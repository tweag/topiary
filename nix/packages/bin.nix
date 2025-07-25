{
  lib,
  stdenv,
  writeShellApplication,

  inotify-tools,
  emscripten,
  git,
  nickel,
  tree-sitter,
  diffutils,
  gnused,
  nixdoc,
  jq,
}:

let
  inherit (builtins)
    readFile
    ;

  # FIXME: Broken
  # TODO: Don't use rustup to install these components but just use Nix
  # generate-coverage = writeShellApplication {
  #   name = "generate-coverage";

  #   runtimeInputs = [
  #     cacert
  #     grcov
  #     rustup
  #   ];

  #   text = readFile ../../bin/generate-coverage.sh;
  # };

  mdbook-generate-nix-documentation = writeShellApplication {
    name = "mdbook-generate-nix-documentation";
    runtimeInputs = [ nixdoc jq ];
    text = readFile ../../bin/mdbook-generate-nix-documentation.sh;
  };

  playground = writeShellApplication {
    name = "playground";

    runtimeInputs = lib.optionals (!stdenv.isDarwin) [
      inotify-tools
    ];

    text = readFile ../../bin/playground.sh;
  };

  update-wasm-app = writeShellApplication {
    name = "update-wasm-app";

    text = readFile ../../bin/update-wasm-app.sh;
  };

  update-wasm-grammars = writeShellApplication {
    name = "update-wasm-grammars";

    runtimeInputs = [
      emscripten
      git
      nickel
      tree-sitter
    ];

    text = readFile ../../bin/update-wasm-grammars.sh;
  };

  verify-documented-usage = writeShellApplication {
    name = "verify-documented-usage";

    runtimeInputs = [
      diffutils
      gnused
    ];

    text = readFile ../../bin/verify-documented-usage.sh;
  };

in
{
  inherit
    mdbook-generate-nix-documentation
    playground
    update-wasm-app
    update-wasm-grammars
    verify-documented-usage
    ;
}
