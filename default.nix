{
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
        ]);
    };
  }
