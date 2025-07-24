{
  lib,
  fetchgit,
  nickel,
  runCommandNoCC,
  writeText,
  tree-sitter,
}:

let
  inherit (builtins)
    attrNames
    concatStringsSep
    mapAttrs
    toFile
    readFile
    toJSON
    fromJSON
    baseNameOf
    ;
  inherit (lib) warn;
  inherit (lib.strings) removeSuffix;
  inherit (lib.attrsets) updateManyAttrsByPath;

  updateByPath = path: update: updateManyAttrsByPath [ { inherit path update; } ];

  /**
    Transforms a JSON-able Nix value into a Nickel file.

    # Type

    ```
    toNickelFile : Any -> File
    ```
  */
  toNickelFile =
    name: e:
    let
      jsonFile = writeText "${removeSuffix ".ncl" name}.json" (toJSON e);
    in
    writeText name "import \"${jsonFile}\"";

  /**
    Converts a JSON-able Nickel file into a Nix value.

    # Type

    ```
    fromNickelFile : File -> Any
    ```
  */
  fromNickelFile =
    path:
    let
      jsonDrv = runCommandNoCC "${removeSuffix ".ncl" (baseNameOf path)}.json" { } ''
        ${nickel}/bin/nickel export ${path} > $out
      '';
    in
    fromJSON (readFile "${jsonDrv}");

in
{
  inherit
    fromNickelFile
    toNickelFile
    ;
}
