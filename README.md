# ssb - Simple status bar

Ssb is a simple status bar for wayland written in rust.

## Features

- [x] Customizable with css
- [x] Hot reload styling
- [ ] Per output configuration
- [ ] Mouse support
- [ ] Hot reload modules

## Modules

- [x] Workspaces (Avaliable only for Hyprland atm)
- [x] Battery
- [x] Backlight
- [x] Pulseaudio
- [x] Memory
- [x] CPU average Load
- [x] Custom scripts
- [ ] Current window title
- [ ] Network
- [ ] Persistant workspaces
- [ ] Bluetooth
- [ ] Tray

## Dependencies

- Compositor implementing the wlr-layer-shell protocol (Hyprland, sway, wayfire, etc.)
- cairo
- libpulseaudio
- rust

## Installation

### The nix way

Include this in your configuration.nix

```nix
    (let
      pkg = import (fetchTarball {
        url = "https://github.com/unixpariah/ssb/archive/main.tar.gz";
      }) {};
    in
      pkg.overrideAttrs (oldAttrs: {
        buildInputs =
          oldAttrs.buildInputs
          ++ [libpulseaudio];
      }))
```

Or if you want specific github revision

```nix
    (let
      pkg = import (fetchGit {
        url = "https://github.com/unixpariah/ssb.git";
        ref = "main";
        rev = "revision hash";
      }) {};
    in
      pkg.overrideAttrs (oldAttrs: {
        buildInputs =
          oldAttrs.buildInputs
          ++ [libpulseaudio];
      }))
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
