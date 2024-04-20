{
  description = "Simple status bar for wayland";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
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
            nodejs_21
            rustc
            cargo
            pkg-config
            cairo
            rustfmt
            rust-analyzer
            clippy
            libpulseaudio
          ];
        };
      in {
        devShell = rustEnv;
        packages = {
          ssb = pkgs.stdenv.mkDerivation {
            name = "ssb";
            src = ./.;
            buildInputs = with pkgs; [rustc cargo];
            buildPhase = ''
              cargo build --release
            '';
            installPhase = ''
              mkdir -p $out/bin
              cp target/release/ssb $out/bin/
            '';
          };
        };
      }
    );
}
