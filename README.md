# ssb - Simple status bar

Ssb is a simple status bar for wayland written in rust.

[![Build Status](https://github.com/unixpariah/ssb/actions/workflows/test.yml/badge.svg)](https://github.com/unixpariah/ssb/actions/workflows/test.yml) [![codecov](https://codecov.io/gh/unixpariah/ssb/graph/badge.svg?token=49LRWZ9D1K)](https://codecov.io/gh/unixpariah/ssb)

## Dependencies

- Compositor implementing the wlr-layer-shell protocol (Hyprland, sway, wayfire, etc.)
- cairo
- libpulseaudio
- rust

## Installation

### The cargo way (Not yet on cargo)

Run this command

```sh
cargo install ssb
```

### The nix way

Include this in your configuration.nix

```nix
    (let
      pkg = import (fetchTarball {
        url = "https://github.com/unixpariah/ssb/archive/master.tar.gz";
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

The configuration file will be generated at XDG_HOME_CONFIG/ssb/config.toml on first run.

## TODO
- [ ] Add sway support (and other compositors as well)
- [ ] Add better styling
- [ ] Optimize modules
- [ ] Add Tray
- [ ] Fix crashing when new output is added
