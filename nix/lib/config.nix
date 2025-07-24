{
  writeShellApplication,
  fromNickelFile,
  toNickelFile,
  prefetchLanguages,
}:

let
  defaultConfigFile = ../../topiary-config/languages.ncl;
  defaultConfig = fromNickelFile defaultConfigFile;
  defaultConfigPrefetched = prefetchLanguages defaultConfig;
  defaultConfigFilePrefetched = toNickelFile "languages-prefetched.ncl" defaultConfigPrefetched;

  wrapWithConfigFile =
    { package, configFile }:
    writeShellApplication {
      name = "topiary";
      text = ''exec ${package}/bin/topiary -C ${configFile} "$@"'';
    };

  wrapWithConfig =
    { package, config }:
    wrapWithConfigFile {
      inherit package;
      configFile = toNickelFile "languages.ncl" config;
    };

in
{
  inherit
    defaultConfig
    defaultConfigPrefetched
    defaultConfigFile
    defaultConfigFilePrefetched
    wrapWithConfig
    wrapWithConfigFile
    ;
}
