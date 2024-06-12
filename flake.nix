{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    forAllSystems = function:
      nixpkgs.lib.genAttrs [
        "x86_64-linux"
        "aarch64-linux"
      ] (system: function nixpkgs.legacyPackages.${system});
  in {
    packages = forAllSystems (pkgs: rec {
      waystatus =
        pkgs.callPackage ./default.nix {
        };
      default = waystatus;
    });

    devShells = forAllSystems (pkgs: {
      default = pkgs.mkShell {
        strictDeps = true;
        nativeBuildInputs = with pkgs; [
          pkg-config
          cargo
          glib
          cairo
          libpulseaudio
          rustc
          rust-analyzer-unwrapped
          rustfmt
          clippy
        ];

        shellHook = ''
          export PKG_CONFIG_PATH=${pkgs.glib.dev}/lib/pkgconfig:${pkgs.cairo.dev}/lib/pkgconfig:${pkgs.libpulseaudio.dev}/lib/pkgconfig
          export NIX_LDFLAGS="-L${pkgs.libpulseaudio.out}/lib"
        '';
      };
    });
  };
}
