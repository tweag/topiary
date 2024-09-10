{ pkgs
, system
, advisory-db
, crane
, rust-overlay
, nix-filter
, craneLib
}:
let
  wasmRustVersion = "1.77.2";
  wasmTarget = "wasm32-unknown-unknown";

  rustWithWasmTarget = pkgs.rust-bin.stable.${wasmRustVersion}.default.override {
    targets = [ wasmTarget ];
  };

  commonArgs = {
    pname = "topiary";

    src = nix-filter.lib.filter {
      root = ./.;
      include = [
        "benches"
        "Cargo.lock"
        "Cargo.toml"
        "languages.ncl"
        "tests"
        "topiary-core"
        "topiary-cli"
        "topiary-config"
        "topiary-playground"
        "topiary-queries"
        "topiary-tree-sitter-facade"
        "topiary-web-tree-sitter-sys"
        "examples"
      ];
    };

    nativeBuildInputs = with pkgs;
      [
        binaryen
        wasm-bindgen-cli
        pkg-config
      ]
      ++ lib.optionals stdenv.isDarwin [
        libiconv
      ];

    buildInputs = with pkgs;
      [
        openssl.dev
      ]
      ++ lib.optionals stdenv.isDarwin [
        darwin.apple_sdk.frameworks.Security
      ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

  # NB: we don't need to overlay our custom toolchain for the *entire*
  # pkgs (which would require rebuidling anything else which uses rust).
  # Instead, we just want to update the scope that crane will use by appending
  # our specific toolchain there.
  craneLibWasm = craneLib.overrideToolchain rustWithWasmTarget;
in
{
  passtru = {
    inherit craneLibWasm;
  };

  clippy = craneLib.cargoClippy (commonArgs
    // {
    inherit cargoArtifacts;
    cargoClippyExtraArgs = "-- --deny warnings";
  });

  clippy-wasm = craneLibWasm.cargoClippy (commonArgs
    // {
    inherit cargoArtifacts;
    cargoClippyExtraArgs = "-p topiary-playground --target ${wasmTarget} -- --deny warnings";
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

  client-app = craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
    pname = "client-app";
    cargoExtraArgs = "-p client-app";
  });

  topiary-core = craneLib.buildPackage (commonArgs
    // {
    inherit cargoArtifacts;
    pname = "topiary-core";
    cargoExtraArgs = "-p topiary-core";
  });

  topiary-cli = craneLib.buildPackage (commonArgs
    // {
    inherit cargoArtifacts;
    pname = "topiary";
    cargoExtraArgs = "-p topiary-cli";
    cargoTestExtraArgs = "--no-default-features";
    postInstall = ''
      install -Dm444 topiary-queries/queries/* -t $out/share/queries
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
      install -Dm444 topiary-queries/queries/* -t $out/share/queries
    '';
  });

  topiary-playground = craneLibWasm.buildPackage (commonArgs
    // {
    inherit cargoArtifacts;
    pname = "topiary-playground";
    cargoExtraArgs = "-p topiary-playground --no-default-features --target ${wasmTarget}";

    # Tests currently need to be run via `cargo wasi` which
    # isn't packaged in nixpkgs yet...
    doCheck = false;

    postInstall = ''
      echo 'Removing unneeded dir'
      rm -rf $out/lib
      echo 'Running wasm-bindgen'
      wasm-bindgen --version
      wasm-bindgen --target web --out-dir $out target/wasm32-unknown-unknown/release/topiary_playground.wasm;
      echo 'Running wasm-opt'
      wasm-opt --version
      wasm-opt -Oz -o $out/output.wasm $out/topiary_playground_bg.wasm
      echo 'Overwriting topiary_playground_bg.wasm with the optimized file'
      mv $out/output.wasm $out/topiary_playground_bg.wasm
      echo 'Extracting custom build outputs'
      export LANGUAGES_EXPORT="$(ls -t target/wasm32-unknown-unknown/release/build/topiary-playground-*/out/languages_export.ts | head -1)"
      cp $LANGUAGES_EXPORT $out/
    '';
  });
}
