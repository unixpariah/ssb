{
  pkgs,
  lib,
  rustPlatform,
}: let
  cargoToml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
  rustPlatform.buildRustPackage {
    pname = "waystatus";
    version = "${cargoToml.package.version}";
    cargoLock.lockFile = ./Cargo.lock;
    src = lib.fileset.toSource {
      root = ./.;
      fileset =
        lib.fileset.intersection
        (lib.fileset.fromSource (lib.sources.cleanSource ./.))
        (lib.fileset.unions [
          ./src
          ./Cargo.toml
          ./Cargo.lock
          ./css-image
        ]);
    };
    nativeBuildInputs = [pkgs.pkg-config pkgs.glib];
    buildInputs = [pkgs.pkg-config];
    configurePhase = ''
      export PKG_CONFIG_PATH=${pkgs.glib.dev}/lib/pkgconfig:${pkgs.cairo.dev}/lib/pkgconfig:${pkgs.libpulseaudio.dev}/lib/pkgconfig
    '';
  }
