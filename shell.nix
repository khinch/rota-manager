# https://wiki.nixos.org/wiki/Rust
 
{
  pkgs ? import <nixpkgs> { config.allowUnfree = true; },
}:
let
  overrides = (builtins.fromTOML (builtins.readFile ./rust-toolchain.toml));
in
pkgs.callPackage (
  {
    stdenv,
    mkShell,
    rust-analyzer,
    rustup,
    rustPlatform,
  }:
  mkShell {
    strictDeps = true;
    # Build time tools (compilers, generators, hooks) available in the dev shell
    nativeBuildInputs = [
      rust-analyzer
      rustup
      rustPlatform.bindgenHook
      pkgs.bruno
      pkgs.pgadmin4-desktopmode
      pkgs.podman
      pkgs.podman-compose
      pkgs.postgresql
      pkgs.sqlx-cli
    ];
    # Libraries and programs needed at runtime, available in the dev shell
    buildInputs =
      [
        pkgs.postman
      ];
    RUSTC_VERSION = overrides.toolchain.channel;
    RUSTUP_TOOLCHAIN = overrides.toolchain.channel;
    # https://github.com/rust-lang/rust-bindgen#environment-variables
    shellHook = ''
      export PATH="''${CARGO_HOME:-~/.cargo}/bin":"$PATH"
      export PATH="''${RUSTUP_HOME:-~/.rustup}/toolchains/$RUSTC_VERSION-${stdenv.hostPlatform.rust.rustcTarget}/bin":"$PATH"
    '';
  }
) { } # Override specific arguments e.g. { rustup = myPinnedRustup; }
