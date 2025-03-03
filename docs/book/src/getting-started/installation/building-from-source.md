# Building from source

Assuming you have the Topiary repository cloned locally, you can build
Topiary in two ways.

## Using Nix

To build Topiary using Nix simply call `nix build` (assuming you have
`flakes` and `nix-command` enabled).

Alternatively, the Topiary flake also has a Topiary package that doesn't
fetch and build the grammars but instead takes them from nixpkgs pinned
by the flake. To build this version use `nix build .#topiary-cli-nix`.

## Using Cargo

Building Topiary using the standard Rust build tools requires not only
those tools, but also some external dependencies. Our flake provides a
devshell that sets all required environment variables and fetches all
dependencies. To enter this devshell use `nix develop` or setup
[direnv].

If you are not a Nix user, you will need to set up all dependencies and
required environment variables. You must ensure at least `pkg-config`
and `openssl` are available.

From there use `cargo build`, as usual.

[direnv]: https://direnv.net/
