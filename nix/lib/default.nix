{
  callPackageNoOverrides,
  topiary-cli,
}:

{
  pre-commit-hook = callPackageNoOverrides ./pre-commit-hook.nix { inherit topiary-cli; };
}
