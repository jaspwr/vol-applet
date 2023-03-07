# Volume Applet
A simple GTK system tray volume controller applet for PulseAudio. Support for PipeWire is planned.

> In it's current state there may be issues running on Wayland and/or with multiple monitors. If you experience any issues, please open an issue.

![Example](https://github.com/jaspwr/vol-applet/blob/main/assets/example.png)

## Installation
### Arch Linux
Clone this repository and install via the PKGBUILD.
### Other Linux
Ensure you have the [dependencies](#dependencies) installed, then run:
```bash
git clone https://github.com/jaspwr/vol-applet
cd vol-applet
cargo build --release
```
The binary will be located at `target/release/volapplet`.

## Usage
```bash
volapplet & disown
```

## Dependencies
* Rust and Cargo
* GTK 3
* PulseAudio