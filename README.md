# Volume Applet
[![AUR](https://img.shields.io/aur/version/volapplet-git)](https://aur.archlinux.org/packages/volapplet-git)

A simple GTK system tray volume controller applet for PulseAudio. Support for PipeWire is planned.

> In it's current state there may be issues running on Wayland and/or with multiple monitors. If you experience any issues, please open an issue.

![Example](https://github.com/jaspwr/vol-applet/blob/main/assets/example.gif)

## Installation
### Arch Linux
Install from the AUR at [volapplet-git](https://aur.archlinux.org/packages/volapplet-git).
### Other Linux
Ensure you have the [dependencies](#dependencies) installed, then run:
```bash
git clone https://github.com/jaspwr/vol-applet
cd vol-applet
cargo build --release
```
The binary will be located at `target/release/volapplet`.

## Usage
Basic usage:
```bash
volapplet & disown
```
Additional functionality can be enabled with arguments:
* `-i` or `--show-inputs`: Add volume controls for inputs.
* `-s` or `--show-streams`: Add volume controls for streams.
* `-d` or `--dont-group`: Don't group controls for inputs and streams into expandable categories.
* `-c` or `--show-icons`: Add icons to each volume control.

Example usage with additional features:
```bash
volapplet -isc & disown
```

## Dependencies
* Rust and Cargo
* GTK 3
* PulseAudio