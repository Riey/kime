# kime

Korean IME

[<img alt="discord" src="https://img.shields.io/discord/801107569505992705.svg?style=for-the-badge" height="25">](https://discord.gg/YPnEfZqC6y)
[<img alt="build status" src="https://img.shields.io/github/workflow/status/Riey/kime/CI/master?style=for-the-badge" height="25">](https://github.com/Riey/kime/actions?query=workflow%3ACI)
[<img alt="release version" src="https://img.shields.io/github/v/release/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/releases)
[<img alt="aur version" src="https://img.shields.io/aur/version/kime?style=for-the-badge" height="25">](https://aur.archlinux.org/packages/kime/)
[<img alt="license" src="https://img.shields.io/github/license/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/blob/master/LICENSE)


## Why kime

* Well tested input engine
* Low memory footprint
* Write in mostly Rust no segfaults
* Allow custom layouts

## Supported frontend

- [x] XIM
- [ ] Wayland
- [x] GTK3
- [ ] GTK4
- [x] Qt5

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

cargo build --release

pkg/release.sh

# You can now install files from build/out
# or use script in pkg/install.sh
# e.g. sudo pkg/install.sh
```

## Configuration

add the following to .xprofile or .xinitrc and restart X:

```sh
export GTK_IM_MODULE=kime
export QT_IM_MODULE=kime
export XMODIFIERS=@im=kime
kime-xim &
```

read [CONFIGURATION.md](CONFIGURATION.md) for detailed options.

## Dependencies

### XIM

* libxcb
* cairo

### GTK3

* gtk3
* pango

### Qt

* qt5gui
