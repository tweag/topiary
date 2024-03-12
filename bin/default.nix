{ pkgs, writeShellApplication }:
{
  generate-coverage = writeShellApplication {
    name = "generate-coverage";

    runtimeInputs = with pkgs; [
      cacert
      grcov
      rustup
    ];

    text = builtins.readFile ./generate-coverage.sh;
  };

  playground = writeShellApplication {
    name = "playground";

    runtimeInputs = with pkgs; [
      inotify-tools
    ];

    text = builtins.readFile ./playground.sh;
  };

  update-wasm-app = writeShellApplication {
    name = "update-wasm-app";

    text = builtins.readFile ./update-wasm-app.sh;
  };

  update-wasm-grammars = writeShellApplication {
    name = "update-wasm-grammars";

    runtimeInputs = with pkgs; [
      emscripten
      git
      toml2json
      tree-sitter
    ];

    text = builtins.readFile ./update-wasm-grammars.sh;
  };

  verify-documented-usage = writeShellApplication {
    name = "verify-documented-usage";

    runtimeInputs = with pkgs; [
      diffutils
      gnused
    ];

    text = builtins.readFile ./verify-documented-usage.sh;
  };
}
