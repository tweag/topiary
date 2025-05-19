{
  fromNickelFile,
  toNickelFile,
  prefetchLanguages,
}:

let
  defaultConfigFile = ../topiary-config/languages.ncl;
  defaultConfig = fromNickelFile defaultConfigFile;
  defaultConfigPrefetched = prefetchLanguages defaultConfig;
  defaultConfigPrefetchedFile = toNickelFile "languages-prefetched.ncl" defaultConfigPrefetched;

in
{
  inherit
    defaultConfig
    defaultConfigFile
    defaultConfigPrefetched
    defaultConfigPrefetchedFile
    ;
}
