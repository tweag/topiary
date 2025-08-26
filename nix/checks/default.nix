{
  emptyFile,
  topiaryPkgs,
  gitHook,
}:

let
  inherit (builtins) deepSeq;

in
{
  # NOTE: The following checks have been removed as WASM
  # and playground development has moved to the playground branch:
  # - clippy-wasm
  # - topiary-playground
  inherit (topiaryPkgs)
    clippy
    fmt
    topiary-core
    audit
    benchmark
    topiary-cli
    ;

  # Check that the `lib.gitHook` output builds/evaluates correctly. `deepSeq e1
  # e2` evaluates `e1` strictly in depth before returning `e2`. We use this
  # trick because checks need to be derivations, which `lib.gitHook` is not.
  gitHook = deepSeq gitHook emptyFile;
}
