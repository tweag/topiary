final: _:

{
  ## A simpler version of `callPackage` that only works on files and does not
  ## rely on `makeOverridable`, to avoid polluting the output.
  callPackageNoOverrides =
    file: args:
    let
      fn = import file;
      auto-args = builtins.intersectAttrs (builtins.functionArgs fn) final;
      final-args = auto-args // args;
    in
    fn final-args;
}
