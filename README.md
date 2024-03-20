# ssb - Simple and hacky status bar

Ssb is a simple status bar for wayland written in rust.

## Dependencies

- Compositor implementing the wlr-layer-shell protocol (Hyprland, sway, wayfire, etc.)
- cairo
- rust

## Installation

1. Clone the repo

```sh
git clone https://github.com/unixpariah/ssb
```

2. Edit configuration in ./ssb/src/config.rs

3. Install dependencies or use nix flake

4. Build the project and install binary

```sh
make install
```

5. Run the binary

```sh
ssb
```

## Configuration

The configuration file will be generated at XDG_HOME_CONFIG/ssb/config.toml on first run.

## TODO
- [ ] Add listener to volume change
- [ ] Fix crashing when new output is added
- [ ] Add proper configuration
- [ ] (Maybe) add x11 support
