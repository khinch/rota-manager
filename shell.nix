# https://wiki.nixos.org/wiki/Rust
 
{
  pkgs ? import <nixpkgs> { config.allowUnfree = true; },
}:
let
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
in
pkgs.callPackage (
  {
    # stdenv,
    mkShell,
    rust-analyzer,
    rustPlatform,
  }:
  mkShell {
    strictDeps = true;

    nativeBuildInputs = [
      rust-analyzer
      rustPlatform.bindgenHook
      pkgs.bruno
      pkgs.cargo
      pkgs.gcc
      pkgs.pgadmin4-desktopmode
      pkgs.pkg-config
      pkgs.podman
      pkgs.podman-compose
      pkgs.postgresql
      pkgs.rustc
      pkgs.sqlx-cli
    ];

    buildInputs = [];

    RUSTC_VERSION = overrides.toolchain.channel;
    # RUSTUP_TOOLCHAIN = overrides.toolchain.channel;
    # https://github.com/rust-lang/rust-bindgen#environment-variables
    shellHook = ''
      export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
    '';
  }
) { } # Override specific arguments e.g. { rustup = myPinnedRustup; }
