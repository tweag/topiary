{ pkgs, nixpkgs, system, advisory-db, crane, rust-overlay, nix-filter }:
let
  rustPkgs = import nixpkgs {
    inherit system;
    overlays = [ (import rust-overlay) ];
  };

  wasmRustVersion = "1.67.1";
  wasmTarget = "wasm32-unknown-unknown";

  rustWithWasmTarget = rustPkgs.rust-bin.stable.${wasmRustVersion}.default.override {
    targets = [ wasmTarget ];
  };

  craneLib = crane.mkLib pkgs;

  commonArgs = {
    src = nix-filter.lib.filter {
      root = ./.;
      include = [
        "benches"
        "Cargo.lock"
        "Cargo.toml"
        "languages"
        "topiary"
        "topiary-cli"
        "topiary-playground"
        "tests"
      ];
    };

    nativeBuildInputs = [
      pkgs.binaryen pkgs.wasm-bindgen-cli
    ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
      pkgs.libiconv
    ];
  };

  cargoArtifacts = craneLib.buildDepsOnly (commonArgs);

  # NB: we don't need to overlay our custom toolchain for the *entire*
  # pkgs (which would require rebuidling anything else which uses rust).
  # Instead, we just want to update the scope that crane will use by appending
  # our specific toolchain there.
  craneLibWasm = craneLib.overrideToolchain rustWithWasmTarget;
in
{
  clippy = craneLib.cargoClippy (commonArgs // {
    inherit cargoArtifacts;
    cargoClippyExtraArgs = "-- --deny warnings";
  });

  clippy-wasm = craneLibWasm.cargoClippy (commonArgs // {
    inherit cargoArtifacts;
    cargoClippyExtraArgs = "-p topiary-playground --target ${wasmTarget} -- --deny warnings";
  });

  fmt = craneLib.cargoFmt (commonArgs);

  audit = craneLib.cargoAudit (commonArgs // {
    inherit advisory-db;
  });

  benchmark = craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
    cargoTestCommand = "cargo bench --profile release";
  });

  topiary-cli = craneLib.buildPackage (commonArgs // {
    inherit cargoArtifacts;
    pname = "topiary";
    cargoExtraArgs = "-p topiary-cli";
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

  topiary-playground = craneLibWasm.buildPackage (commonArgs // {
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
    '';
  });
}
