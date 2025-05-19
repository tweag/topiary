{
  writeShellApplication,

  fromNickelFile,
  toNickelFile,
  prefetchLanguages,
}:

let
  defaultConfigFile = ../topiary-config/languages.ncl;
  defaultConfig = fromNickelFile defaultConfigFile;
  defaultConfigPrefetched = prefetchLanguages defaultConfig;
  defaultConfigPrefetchedFile = toNickelFile "languages-prefetched.ncl" defaultConfigPrefetched;

  wrapWithConfigFile =
    topiary: topiaryConfigFile:
    writeShellApplication {
      name = "topiary";
      text = ''exec ${topiary}/bin/topiary -C ${topiaryConfigFile} "$@"'';
    };

  wrapWithConfig =
    topiary: topiaryConfig: wrapWithConfigFile topiary (toNickelFile "languages.ncl" topiaryConfig);

in
{
  inherit
    defaultConfig
    defaultConfigFile
    defaultConfigPrefetched
    defaultConfigPrefetchedFile
    wrapWithConfig
    wrapWithConfigFile
    ;
}
