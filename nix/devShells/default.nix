{
  callPackage,
  checks,
  craneLib,
  binPkgs,
  topiaryPkgs,
}:

{
  default = callPackage ./devShell.nix {
    inherit craneLib binPkgs checks;
  };

  minimal = callPackage ./devShell.nix {
    checks = {
      inherit (checks)
        clippy
        fmt
        topiary-core
        topiary-cli
        ;
    };
    inherit craneLib binPkgs;
    includeExtraPackages = false;
  };

  wasm = callPackage ./devShell.nix {
    inherit binPkgs checks;
    craneLib = topiaryPkgs.clippy-wasm.passthru.craneLibWasm;
  };
}
