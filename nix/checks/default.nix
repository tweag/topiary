{
  hello,
  topiaryPkgs,
  pre-commit-hook,
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

  ## Check that the `lib.pre-commit-hook` output builds/evaluates
  ## correctly. `deepSeq e1 e2` evaluates `e1` strictly in depth before
  ## returning `e2`. We use this trick because checks need to be
  ## derivations, which `lib.pre-commit-hook` is not.
  pre-commit-hook = deepSeq pre-commit-hook hello;
}
