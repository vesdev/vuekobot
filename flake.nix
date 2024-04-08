{
  inputs = {
    nixpkgs.url = "nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    nci.url = "github:yusdacra/nix-cargo-integration";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, flake-parts, nixpkgs, rust-overlay, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" ];
      imports = [ inputs.nci.flakeModule ];

      perSystem = { config, pkgs, system, ... }:
        let
          devInputs = with pkgs; [ diesel-cli python3Packages.pgcli ];
          crate = config.nci.outputs.vuekobot;
        in {
          nci.projects.vuekobot.path = ./.;

          nci.crates.vuekobot = {
            export = true;
            drvConfig.mkDerivation = {
              nativeBuildInputs = with pkgs; [ pkg-config openssl ];
              buildInputs = with pkgs; [ libpqxx postgresql.lib ];
            };
          };

          nci.toolchains.shell =
            (rust-overlay.packages.${system}.rust.override {
              extensions = [
                "cargo"
                "clippy"
                "rust-src"
                "rust-analyzer"
                "rustc"
                "rustfmt"
              ];
            });
          devShells.default = crate.devShell.overrideAttrs
            (old: { packages = (old.packages or [ ]) ++ devInputs; });

          packages = rec {
            vuekobot = crate.packages.release;
            default = vuekobot;
          };
        };

      flake.nixosModules = rec {
        vuekobot = import ./module.nix;
        default = vuekobot;
      };
    };
}
