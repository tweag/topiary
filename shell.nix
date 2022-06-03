{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
    nativeBuildInputs = with pkgs; [ rustc cargo gcc ];
    buildInputs = with pkgs; [ rustfmt clippy rust-analyzer ];

    RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
}
