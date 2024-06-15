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
        nativeBuildInputs = with pkgs; [
          pkg-config
          cargo
          cairo
          libpulseaudio
          rustc
          rust-analyzer
          rustfmt
          clippy
        ];
      };
    });
  };
}
