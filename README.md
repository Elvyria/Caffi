# Caffi
[![Crates.io](https://img.shields.io/crates/v/caffi?logo=rust)](https://crates.io/crates/caffi)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow)](https://opensource.org/licenses/MIT)

Caffi is a simple desktop calendar that is in a very early stage of development.

![Preview](https://github.com/user-attachments/assets/ede121d5-67c0-4019-beda-5c25c96d64b8)


## Installation
Can be installed from [crates.io](https://crates.io/) with `cargo`:

```sh
cargo install caffi --locked --features Sass,Wayland...
```

## Dependencies
* [GTK4](https://www.gtk.org/)
* [gtk4-layer-shell](https://github.com/wmww/gtk4-layer-shell) (Feature: Wayland)
* [libxcb](https://xcb.freedesktop.org/) (Feature: X11)

## Features
Some features can be enabled at compile time.
* [Accent](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.Settings.html) - Inherits the accent color from the system's settings.
* [Sass](https://sass-lang.com/) - Allows you to use SCSS instead of CSS.
* [Wayland](https://wayland.freedesktop.org/) - Uses wlr-layer-shell to imitate window positioning.
* [X11](https://www.x.org/) - Sets WM hints and properties, and repositions the window.

## Usage
```
Usage: caffi [-1 <first>] [-a <anchor...>] [-m <margin...>] [-u <userstyle>] [-v]

Calendar

Options:
  -1, --first       first day of the week: (sun)day, (mon)day, (tue)sday...
  -a, --anchor      screen anchor point: (t)op, (b)ottom, (l)eft, (r)ight
  -m, --margin      margin distance for each anchor point
  -u, --userstyle   path to the userstyle
  -v, --version     print version
  --help            display usage information
```

## Customization
Caffi is built with GTK4 and uses CSS to define its appearance.  
You will find the style sheet in your config directory after the first launch.
```sh
${XDG_CONFIG_HOME:-$HOME/.config}/caffi/style.css
```
If you have enabled the Sass feature, it will also look for *.scss and *.sass files.
```sh
${XDG_CONFIG_HOME:-$HOME/.config}/caffi/style.sass
${XDG_CONFIG_HOME:-$HOME/.config}/caffi/style.scss
```

## Tips
### Anchoring
It is often desirable to be able to position widgets relatively to a screen side.  
Two flags will help with this: `-a --anchor` and `-m --margin`.  
Each margin value provided will match every anchor point respectively.  
```sh
caffi --anchor left --anchor bottom --margin 20 --margin 30
```

### Toggle Window
If you want to toggle window with a click of a button, Unix way is the way:
```sh
pkill caffi | caffi
```

## Troubleshooting

### Environment
Caffi is developed and tested with:
* Wayland (Hyprland): `0.45.2`

If your setup is different and you experience issues, feel free to file a bug report.

### GTK
To get GTK related messages a specific environment variable must be non empty.
```sh
GTK_DEBUG=1 caffi
```

## Building
To build this little thing, you'll need some [Rust](https://www.rust-lang.org/).

```sh
git clone --depth 1 https://github.com/Elvyria/caffi
cd caffi
cargo build --locked --release --features Sass,Wayland...
```
