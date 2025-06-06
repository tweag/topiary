{
  pkgs,
  topiary-cli,
}:

{
  enable = true;
  name = "topiary";
  description = "A general code formatter based on tree-sitter.";
  entry = "${pkgs.lib.getExe topiary-cli} fmt";
  types = [ "text" ];
}
