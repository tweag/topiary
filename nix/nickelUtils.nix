{
  lib,
  nickel,
  runCommandNoCC,
  writeText,
}:

let
  inherit (builtins)
    readFile
    toJSON
    fromJSON
    baseNameOf
    ;
  inherit (lib.strings)
    removeSuffix
    ;

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
    toNickelFile
    fromNickelFile
    ;
}
