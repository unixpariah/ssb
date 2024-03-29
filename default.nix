{pkgs ? import <nixpkgs> {}}: let
  manifest = (pkgs.lib.importTOML ./Cargo.toml).package;
in
  pkgs.rustPlatform.buildRustPackage rec {
    pname = manifest.name;
    version = manifest.version;
    cargoLock.lockFile = ./Cargo.lock;
    src = pkgs.lib.cleanSource ./.;

    buildInputs = with pkgs; [
      cairo
      libpulseaudio
    ];

    nativeBuildInputs = with pkgs; [
      pkg-config
    ];

    NIX_LDFLAGS = "-L${pkgs.libpulseaudio.out}/lib";
  }
