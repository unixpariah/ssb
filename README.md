# Waystatus

Waystatus is a simple status bar for wlroots based compositors.

## Features

- [x] Customizable with css
- [x] Hot reload styling
- [ ] Per output configuration
- [ ] Hot reload modules
- [ ] Mouse support

## Modules

- [x] Workspaces:
    - [x] Hyprland
    - [x] Sway
- [x] Persistant workspaces
    - [x] Hyprland
    - [x] Sway
- [x] Current window title
    - [x] Hyprland
    - [x] Sway
- [x] Battery
- [x] Backlight
- [x] Pulseaudio
- [x] Memory
- [x] CPU average Load
- [x] Custom scripts
- [ ] Network
- [ ] Date and time
- [ ] Bluetooth

## Build time dependencies

- rust
- cairo
- libpulseaudio

## Installation

### The nix way

Include this in your configuration.nix

```nix
    (let
      waystatus =
        import (pkgs.fetchgit {
        url = "https://github.com/unixpariah/waystatus.git";
        rev = "e9e558d3d17d4e95b934ad9a1a26a686370cc6de";
        sha256 = "0r77vi12a1wng17hd76v6s867kv38g8rijqqq7yrb5pmb37ncszy";
        fetchSubmodules = true;
    }) {pkgs = pkgs;};
    in
      waystatus
    )
```

### The manual way

1. Clone the repository and cd into it

```sh
git clone https://github.com/unixpariah/waystatus.git && cd waystatus
```

2. Install necessary dependencies or use nix flake

3. Build the project

```sh
cargo build --release
```

4. Run the binary

```sh
./target/release/waystatus
```

## Configuration

The configuration files will be generated at XDG_HOME_CONFIG/waystatus/* on first run.

Styling with css is handled by [css-image](https://github.com/unixpariah/css-image) and is currently very limited.
