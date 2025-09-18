with (import <nixpkgs> { config.allowUnfree = true; });
mkShell{
  buildInputs = [
    pgadmin4-desktopmode
    podman
    podman-compose
    postgresql
    postman
    protobuf
    sqlx-cli
  ];
}
