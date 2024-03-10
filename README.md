# ssb - Simple and hacky status bar

Ssb is a simple status bar for wayland (with plans of adding x11 support) written in rust.

## Dependencies

- Compositor implementing the wlr-layer-shell protocol (Hyprland, sway, wayfire, etc.)
- rust

## Installation

1. Clone the repo

```sh
git clone https://github.com/unixpariah/ssb
```

2. Edit configuration in ./ssb/src/config.rs

3. Build the project and install binary

```sh
make install
```

4. Run the binary

```sh
ssb
```

## Configuration

The configuration file is located in ./ssb/src/config.rs. Ssb is supposed to be simple and minimalistic so configuration is done by editing the source code.
After editing the configuration file, you need to rebuild the project and reinstall the binary.
