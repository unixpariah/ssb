# ssb - Simple status bar

Ssb is a simple status bar for wlroots based compositors.

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

- cairo
- libpulseaudio
- rust

## Installation

### The nix way

Include this in your configuration.nix

```nix
    (let
      ssb =
        import (pkgs.fetchgit {
        url = "https://github.com/unixpariah/ssb.git";
        rev = "3c29f7402295deee9540ce3a5317f3757a9d0932";
        sha256 = "0dhx501c0gwdz64k58n0471pnkq1yjpmdvdvcydpicggbkvk9pg0";
        fetchSubmodules = true;
    }) {pkgs = pkgs;};
    in
      ssb
    )
```

### The manual way

1. Clone the repository and cd into it

```sh
git clone https://github.com/unixpariah/ssb.git && cd ssb
```

2. Install necessary dependencies or use nix flake

3. Build the project

```sh
cargo build --release
```

4. Run the binary

```sh
./target/release/ssb
```

## Configuration

The configuration files will be generated at XDG_HOME_CONFIG/ssb/* on first run.

Styling with css is handled by [css-image](https://github.com/unixpariah/css-image) and is currently very limited.
