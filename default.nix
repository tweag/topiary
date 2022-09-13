{ pkgs, crane }: 
let
  craneLib = crane.mkLib pkgs;
in
{
  app = craneLib.buildPackage {
    src = ./.;
    nativeBuildInputs = [ pkgs.libiconv ];
  };
}