# Using with Nix
Topiary provides a flake with several attributes. The main one is `topiary-cli`
that produces a version of the CLI that doesn't come with any tree-sitter
grammars. However, this version cannot be used in Nix. For that purpose the
flake also provides the `topiary-cli-nix` package. This package utilizes the
tree-sitter grammars from the `nixpkgs` flake input. Note that the tree-sitter
grammar for OCamlLex hasn't been added to nixpkgs yet, and so this build
disables support for that language.

## Git Hooks
Topiary integrates seamlessly with [pre-commit-hooks.nix]: add Topiary as input
to your flake and, in [pre-commit-hooks.nix]'s setup, use:

``` nix
pre-commit-check = nix-pre-commit-hooks.run {
  hooks = {
    nixfmt.enable = true; ## keep your normal hooks
    ...
    ## Add the following:
    topiary = topiary.lib.${system}.pre-commit-hook;
  };
};
```

[pre-commit-hooks.nix]: https://github.com/cachix/pre-commit-hooks.nix
