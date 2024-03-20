{
  description = "Simple status bar for wayland";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = import nixpkgs {
          inherit system;
        };
        rustEnv = pkgs.mkShell {
          buildInputs = with pkgs; [
            rustc
            cargo
            pkg-config
            cairo
            rustfmt
            rust-analyzer
            clippy
          ];
        };
      in {
        devShell = rustEnv;
        packages = {
          my-rust-app = pkgs.stdenv.mkDerivation {
            name = "my-rust-app";
            src = ./.;
            buildInputs = with pkgs; [rustc cargo];
            buildPhase = ''
              cargo build --release
            '';
            installPhase = ''
              mkdir -p $out/bin
              cp target/release/my-rust-app $out/bin/
            '';
          };
        };
      }
    );
}
