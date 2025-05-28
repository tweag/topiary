{
  writeShellApplication,

  fromNickelFile,
  toNickelFile,
  prefetchLanguages,
}:

let
  /**
    ```
    defaultConfigFile : File
    ```

    Same as `defaultConfig` but as a Nickel File.
  */
  defaultConfigFile = ../topiary-config/languages.ncl;

  /**
    ```
    defaultConfig : TopiaryConfig
    ```

    The default configuration of Topiary -- that is the content of
    `topiary-config/languages.ncl` -- but as a Nix value.
  */
  defaultConfig = fromNickelFile defaultConfigFile;

  /**
    ```
    defaultConfigPrefetched
    ```

    Same as `defaultConfig`, but where all the language grammars point to the
    Nix store path of a prefetched and precompiled version of the grammar. This
    is `prefetchLanguages defaultConfig`.
  */
  defaultConfigPrefetched = prefetchLanguages defaultConfig;

  /**
    ```
    defaultConfigPrefetchedFile : File
    ```

    Same as `defaultConfigPrefetched` but as a Nickel file.
  */
  defaultConfigPrefetchedFile = toNickelFile "languages-prefetched.ncl" defaultConfigPrefetched;

  /**
    ```
    wrapWithConfigFile : Derivation -> File -> Derivation
    ```

    Same as `wrapWithConfig` but with the configuration as a Nickel file.
  */
  wrapWithConfigFile =
    topiary: topiaryConfigFile:
    writeShellApplication {
      name = "topiary";
      text = ''exec ${topiary}/bin/topiary -C ${topiaryConfigFile} "$@"'';
    };

  /**
    ```
    wrapWithConfig : Derivation -> TopiaryConfig -> Derivation
    ```

    Given a derivation providing `/bin/topiary` and a Topiary configuration
    `<config>`, produce an executable running `topiary -C <config>`.

    Note that this is different from building Topiary with a different default
    configuration: the resulting binary will not accept an additional `-C`
    argument.
  */
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
