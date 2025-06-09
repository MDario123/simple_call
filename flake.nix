{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        config.allowUnfree = true;
        inherit system;
      };
    in
    {
      packages."${system}" = {

        server = pkgs.rust.packages.stable.rustPlatform.buildRustPackage {
          name = "server";
          src = ./server;

          cargoLock = {
            lockFile = ./server/Cargo.lock;
          };
        };

        tty_client = pkgs.rust.packages.stable.rustPlatform.buildRustPackage {
          name = "tty_client";
          src = ./tty_client;

          cargoLock = {
            lockFile = ./tty_client/Cargo.lock;
          };

          nativeBuildInputs = with pkgs; [
            pkg-config
            alsa-lib
          ];
        };

      };

      devShells."${system}" = {
        android_app = pkgs.mkShell {
          buildInputs = with pkgs; [
            android-studio
          ];

          shellHook = ''
            export SHELL=$(which zsh)
          '';
        };

        tty_client = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
            rust-analyzer

            pkg-config
            cmake
            alsa-lib
            libopus
          ];

          shellHook = ''
            export SHELL=$(which zsh)
          '';
        };
      };
    };
}
