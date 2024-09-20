# Building from source
Assuming you have the Topiary repository cloned locally, you can build Topiary in two ways.

## Using Nix
To build Topiary using nix simply call `nix build`, this assumes you have
`flakes` and `nix-command` enabled.

Alternatively, the Topiary flake also has a Topiary package that doesn't fetch
and build the grammars but instead takes them from the nixpkgs pinned by the
flake. To build this version use `nix build .#topiary-cli-nix`.

## Using Cargo
Building Topiary using the standard rust build tools requires not only those
tools, but also some external dependencies. Our flake provides a devshell that
sets all required environment variables and fetches all dependencies. To enter
this devshell use `nix develop` or setup [direnv][direnv].

If you cannot/do not want to use Nix, you are responsible for getting all
dependencies and setting the required environment variables. You must ensure at
least `pkg-config` and `openssl` are available.
  
From there use `cargo build` to build `topiary-cli`.

[direnv]: https://direnv.net/
