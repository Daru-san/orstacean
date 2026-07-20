{
  description = "Tracing back one's origin, the crabby way.";

  inputs = {
    flake-parts.url = "github:hercules-ci/flake-parts";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "x86_64-linux"
        "aarch64-linux"
        "aarch64-darwin"
        "x86_64-darwin"
      ];
      perSystem =
        {
          pkgs,
          lib,
          ...
        }:
        let
          fs = lib.fileset;
        in
        {
          packages.default = pkgs.rustPlatform.buildRustPackage {
            pname = "orstacean";
            version = "0.1";
            src = fs.toSource {
              root = ./.;
              fileset = fs.intersection (fs.fromSource (lib.sources.cleanSource ./.)) (
                fs.unions [
                  ./src
                  ./Cargo.toml
                  ./Cargo.lock
                  ./assets
                ]
              );
            };

            cargoLock = {
              lockFile = ./Cargo.lock;
              outputHashes = {
                "ratatui-form-0.1.1" = "sha256-IGYUhY9H61J77vecEvItVce5lTy00OUf+0VTIvdBV3I=";
              };
            };

            nativeBuildInputs = with pkgs; [
              pkg-config
              wild
            ];

            buildInputs = with pkgs; [
              chafa
              glib
              alsa-lib
            ];

            meta = {
              maintainers = [ lib.maintainers.Daru-san ];
              mainProgram = "orstacean";
              license = lib.licenses.bsd3;
            };
          };
          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              cargo
              rustc
              rustfmt
              chafa
              pkg-config
              glib
              wild
              alsa-lib
              clippy
            ];
            RUST_SRC_PATH = pkgs.rustPlatform.rustLibSrc;
          };
        };
    };
}
