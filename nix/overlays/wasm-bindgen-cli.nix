final: _:

let
  inherit (builtins)
    elemAt
    filter
    fromTOML
    length
    map
    readFile
    ;

  cargoLock = fromTOML (readFile ../../Cargo.lock);
  wasmBindgenCargoVersions = map ({ version, ... }: version) (
    filter ({ name, ... }: name == "wasm-bindgen") cargoLock.package
  );
  wasmBindgenVersion =
    assert length wasmBindgenCargoVersions == 1;
    elemAt wasmBindgenCargoVersions 0;

in
{
  wasm-bindgen-cli = final.buildWasmBindgenCli rec {
    src = final.fetchCrate {
      pname = "wasm-bindgen-cli";
      version = wasmBindgenVersion;
      hash = "sha256-txpbTzlrPSEktyT9kSpw4RXQoiSZHm9t3VxeRn//9JI=";
    };

    cargoDeps = final.rustPlatform.fetchCargoVendor {
      inherit src;
      pname = "${src.pname}-${src.version}";
      hash = "sha256-J+F9SqTpH3T0MbvlNKVyKnMachgn8UXeoTF0Pk3Xtnc=";
    };
  };
}
