{ pkgs, writeShellApplication }:

let
  inherit (builtins)
    readFile
    ;

  inherit (pkgs.lib)
    optionals
    ;

  # FIXME: Broken
  # TODO: Don't use rustup to install these components but just use Nix
  # generate-coverage = writeShellApplication {
  #   name = "generate-coverage";

  #   runtimeInputs = with pkgs; [
  #     cacert
  #     grcov
  #     rustup
  #   ];

  #   text = readFile ../../bin/generate-coverage.sh;
  # };

  playground = writeShellApplication {
    name = "playground";

    runtimeInputs =
      with pkgs;
      optionals (!stdenv.isDarwin) [
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

    runtimeInputs = with pkgs; [
      emscripten
      git
      nickel
      tree-sitter
    ];

    text = readFile ../../bin/update-wasm-grammars.sh;
  };

  verify-documented-usage = writeShellApplication {
    name = "verify-documented-usage";

    runtimeInputs = with pkgs; [
      diffutils
      gnused
    ];

    text = readFile ../../bin/verify-documented-usage.sh;
  };

in
{
  inherit
    playground
    update-wasm-app
    update-wasm-grammars
    verify-documented-usage
    ;
}
