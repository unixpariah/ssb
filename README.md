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

## Building

1. Install dependencies:

- rust
- cairo
- libpulseaudio

2. Clone the repository and cd into it

```sh
git clone https://github.com/unixpariah/waystatus.git && cd waystatus
```

3. Build the project

```sh
cargo build --release
```

## Configuration

The configuration files will be generated at XDG_HOME_CONFIG/waystatus/* on first run.

Styling with css is handled by [css-image](https://github.com/unixpariah/css-image) and is currently very limited.
