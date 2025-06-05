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

  light = callPackage ./devShell.nix {
    checks = # checksLight
      {
        inherit (topiaryPkgs)
          clippy
          fmt
          topiary-core
          topiary-cli
          ;
      };
    inherit craneLib binPkgs;
    optionals = false;
  };

  wasm = callPackage ./devShell.nix {
    inherit binPkgs checks;
    craneLib = topiaryPkgs.clippy-wasm.passthru.craneLibWasm;
  };
}
