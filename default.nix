{ pkgs, advisory-db, crane, nix-filter }:
let
  craneLib = crane.mkLib pkgs;

  commonArgs = {
    src = nix-filter.lib.filter {
      root = ./.;
      include = [
        "benches"
        "Cargo.lock"
        "Cargo.toml"
        "languages"
        "src"
        "tests"
      ];
    };

    nativeBuildInputs = [ pkgs.libiconv ];
  };

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs);
in
{
  clippy = craneLib.cargoClippy (commonArgs // {
    inherit cargoArtifacts;
    cargoClippyExtraArgs = "--all-targets -- --deny warnings";
  });

  fmt = craneLib.cargoFmt (commonArgs);

  audit = craneLib.cargoAudit (commonArgs // {
    inherit advisory-db;
  });

  benchmark = craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
    cargoTestCommand = "cargo bench --profile release";
  });

  app = craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
    postInstall = ''
      install -Dm444 languages/* -t $out/share/languages
    '';

    # Set TOPIARY_LANGUAGE_DIR to the Nix store
    # for the build
    TOPIARY_LANGUAGE_DIR = "${placeholder "out"}/share/languages";

    # Set TOPIARY_LANGUAGE_DIR to the working directory
    # in a development shell
    shellHook = ''
      export TOPIARY_LANGUAGE_DIR=$PWD/languages
    '';
  });
}
