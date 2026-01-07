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

### Display Options

- `os_art` - Show OS-specific art instead of the Slowfetch logo. Set to `true` for auto-detect or specify an OS name
- `custom_art` - Path to custom ASCII art file. Supports color placeholders `{0}` through `{7}`
- `image` - Display an image instead of ASCII art (requires Kitty graphics protocol)
- `image_path` - Custom image path when using image mode
- `nerd_fonts` - Force nerd font icons on or off (overrides auto-detection)
- `box_style` - Box corner style: `"rounded"` or `"square"`
- `border_line_style` - Border line style: `"solid"`, `"dotted"`, or `"double"`

### Color Themes

Set a theme preset or customize individual colors using web hex format.

Available themes: `tty`, `dracula`, `catppuccin`, `nord`, `gruvbox`, `eldritch`, `kanagawa`, `rosepine`

Individual color options: `border`, `title`, `key`, `value`, and `art_1` through `art_8` for ASCII art colors.

### Section Toggles

Toggle which items to show in each section:

- Core: `os`, `kernel`, `uptime`, `init`
- Hardware: `cpu`, `gpu`, `memory`, `storage`, `battery`, `screen`
- Userspace: `packages`, `terminal`, `shell`, `wm`, `ui`, `editor`, `terminal_font`

The default config can be found in `src/config.toml`.

## Contributing

I currently won't accept PRs as this defeats the whole point of the project (sorry!).

## Installation

To install Slowfetch, pull the source and use the following command from the root of the project:

```
cargo install --path .
```

## Example

![Slowfetch Screenshot](https://raw.githubusercontent.com/tuibird/Slowfetch/refs/heads/master/slowfetch/screenshot1.png)

![Slowfetch TUI config](https://raw.githubusercontent.com/tuibird/Slowfetch/refs/heads/master/slowfetch/screenshot2.png)

![Slowfetch image display](https://raw.githubusercontent.com/tuibird/Slowfetch/refs/heads/master/slowfetch/screenshot3.png)