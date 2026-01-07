![Slowfetch Logo](https://raw.githubusercontent.com/tuibird/Slowfetch/refs/heads/master/slowfetch.png)

A fetch program for my girlfriend. She doesnt rice, so the goal was a fetch program that looks riced for her out of the box.

I'm mainly doing this for the sake of learning and honestly making a fetch program with "pretty" defaults sounds good to me.
As far as the hardware it supports, I run CachyOS which is Arch based and Gentoo. So its primarily designed around my setups. Expect weirdness as I iron out bugs with OS/Hardware support.
I will be adding features as I go, but this is by no means stable. Expect some breaking changes, even in main.

## Command Line Arguments

| Argument | Description |
|----------|-------------|
| `-o, --os [name]` | Display OS art instead of the Slowfetch logo. Optionally specify an OS name to force that logo (e.g. `--os arch`) |
| `-i, --image [path]` | Display an image instead of ASCII art using Kitty graphics protocol. Optionally specify image path |
| `-c, --config` | Launch the TUI configuration editor |
| `-r, --refresh` | Force refresh of cached values (OS name and GPU) |
| `-u, --update` | Update config file to latest version while preserving user settings |

## Configuration
Configuration can be done either through the TUI interface or in the config file.

The config file is located at `~/.config/slowfetch/config.toml` and is created automatically on first run.

The default config can also be found in `src/config.toml`.

## Contributing

I currently won't accept PRs as this defeats the whole point of the project (sorry!).

## Installation

To install Slowfetch, pull the source and use the following command from the root of the project:

```
cargo install --path .
```

## Screenshots

![Slowfetch Screenshot](https://raw.githubusercontent.com/tuibird/Slowfetch/refs/heads/master/screenshot1.png)

![Slowfetch TUI config](https://raw.githubusercontent.com/tuibird/Slowfetch/refs/heads/master/screenshot2.png)

![Slowfetch image display](https://raw.githubusercontent.com/tuibird/Slowfetch/refs/heads/master/screenshot3.png)