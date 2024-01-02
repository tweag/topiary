{ pkgs
, system
, advisory-db
, crane
, rust-overlay
, nix-filter
,
}:
let
  craneLib = crane.mkLib pkgs;

  commonArgs = {
    pname = "topiary";

    src = nix-filter.lib.filter {
      root = ./.;
      include = [
        "benches"
        "Cargo.lock"
        "Cargo.toml"
        "languages.toml"
        "queries"
        "topiary"
        "topiary-queries"
        "topiary-cli"
        "tests"
      ];
    };

    nativeBuildInputs = with pkgs;
      [
        binaryen
      ]
      ++ lib.optionals stdenv.isDarwin [
        libiconv
      ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;
in
{
  clippy = craneLib.cargoClippy (commonArgs
    // {
    inherit cargoArtifacts;
    cargoClippyExtraArgs = "-- --deny warnings";
  });

  fmt = craneLib.cargoFmt commonArgs;

  audit = craneLib.cargoAudit (commonArgs
    // {
    inherit advisory-db;
  });

  benchmark = craneLib.buildPackage (commonArgs
    // {
    inherit cargoArtifacts;
    cargoTestCommand = "cargo bench --profile release";
  });

  topiary-lib = craneLib.buildPackage (commonArgs
    // {
    inherit cargoArtifacts;
    pname = "topiary-lib";
    cargoExtraArgs = "-p topiary";
  });

  topiary-cli = craneLib.buildPackage (commonArgs
    // {
    inherit cargoArtifacts;
    pname = "topiary";
    cargoExtraArgs = "-p topiary-cli";
    postInstall = ''
      install -Dm444 queries/* -t $out/share/queries
    '';

    # Set TOPIARY_LANGUAGE_DIR to the Nix store
    # for the build
    TOPIARY_LANGUAGE_DIR = "${placeholder "out"}/share/queries";

    # Set TOPIARY_LANGUAGE_DIR to the working directory
    # in a development shell
    shellHook = ''
      export TOPIARY_LANGUAGE_DIR=$PWD/queries
    '';
  });

  topiary-queries = craneLib.buildPackage (commonArgs
    // {
    pname = "topiary-queries";
    cargoExtraArgs = "-p topiary-queries";
    postInstall = ''
      install -Dm444 queries/* -t $out/share/queries
    '';
  });
}
