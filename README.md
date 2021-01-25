# kime

Korean IME

[<img alt="discord" src="https://img.shields.io/discord/801107569505992705.svg?style=for-the-badge" height="25">](https://discord.gg/YPnEfZqC6y)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/Riey/kime/CI/master?style=for-the-badge" height="25">](https://github.com/Riey/kime/actions?query=workflow%3ACI)
[<img alt="release version" src="https://img.shields.io/github/v/release/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/releases)
[<img alt="aur version" src="https://img.shields.io/aur/version/kime?style=for-the-badge" height="25">](https://aur.archlinux.org/packages/kime/)
[<img alt="license" src="https://img.shields.io/github/license/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/blob/master/LICENSE)

## [Changelog](docs/CHANGELOG.md)

## Why kime

* Well tested input engine
* Low memory footprint
* Write in mostly Rust no segfaults
* Allow custom layouts

## Supported frontend

- [x] XIM
- [ ] Wayland
- [x] GTK2
- [x] GTK3
- [x] GTK4
- [x] Qt5
- [x] Qt6

## Installation

### Arch Linux

you can install from AUR package [kime](https://aur.archlinux.org/packages/kime) for latest release, or [kime-git](https://aur.archlinux.org/packages/kime-git) if you want to build from source.

### Debian

you can install from .deb file at [releases](https://github.com/Riey/kime/releases) tab.

### Build from source

make sure **cargo** and other dependencies listed below are installed before build.

```sh
git clone https://github.com/Riey/kime
cd kime

cargo xtask build XIM GTK3 QT5

# You can now install files from build/out
# or use install task
# cargo xtask install <target-path>
# or you are debian user, use release-deb
# cargo xtask release-deb <deb-out-path>
```

See `cargo xtask --help` for more detail


#### GTK

```sh
# If you install gtk2
sudo gtk-query-immodules-2.0 --update-cache
# If you install gtk3
sudo gtk-query-immodules-3.0 --update-cache
# If you install gtk4
sudo gio-querymodules /usr/lib/gtk-4.0/4.0.0/immodules
```

## Configuration

add the following to .xprofile or .xinitrc and restart X:

if you don't use XIM, you don't have to run `kime-xim`

```sh
export GTK_IM_MODULE=kime
export QT_IM_MODULE=kime
export XMODIFIERS=@im=kime
kime-xim &
```

read [CONFIGURATION.md](docs/CONFIGURATION.md) for detailed options.

## Dependencies

### XIM

* libxcb
* cairo

### Other specific toolkit immodule

* that toolkit(e.g. gtk3, qt5 ...)
