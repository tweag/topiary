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

  toNickelFile =
    name: e:
    let
      jsonFile = writeText "${removeSuffix ".ncl" name}.json" (toJSON e);
    in
    writeText name "import \"${jsonFile}\"";

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
