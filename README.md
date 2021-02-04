# kime

Korean IME

[English](./README.md), [한국어](./README.ko.md)

[<img alt="build" src="https://img.shields.io/github/workflow/status/Riey/kime/CI?style=for-the-badge" height="25">](https://github.com/Riey/kime/actions?query=workflow%3ACI)
[<img alt="discord" src="https://img.shields.io/discord/801107569505992705.svg?style=for-the-badge" height="25">](https://discord.gg/YPnEfZqC6y)
[<img alt="release version" src="https://img.shields.io/github/v/release/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/releases)
[<img alt="aur version" src="https://img.shields.io/aur/version/kime?style=for-the-badge" height="25">](https://aur.archlinux.org/packages/kime/)
[<img alt="license" src="https://img.shields.io/github/license/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/blob/master/LICENSE)
[<img src="https://d1u4yishnma8v5.cloudfront.net/mobile-gift.png" alt="donaricano-btn" height="50">](https://donaricano.com/mypage/1610220543_mjZDXO)

## [Changelog](docs/CHANGELOG.md)

## Why kime

* Well tested input engine
* Low memory footprint
* Write in mostly Rust no segfaults
* Allow custom layouts

## Supported frontend

- [x] XIM
- [x] Wayland
- [x] GTK2
- [x] GTK3
- [x] GTK4
- [x] Qt5
- [x] Qt6

## Installation

### Arch Linux

you can install from AUR package [kime](https://aur.archlinux.org/packages/kime) for latest release
or [kime-git](https://aur.archlinux.org/packages/kime-git) if you want to build from source.

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

add the following to your init script

```sh
export GTK_IM_MODULE=kime
export QT_IM_MODULE=kime
export XMODIFIERS=@im=kime
```

and run `kime-xim` or `kime-wayland` binary after session initialized

if you use X it could be done in .xprofile

also run `kime-indicator` when you want show hangul status with appindicator

read [CONFIGURATION.md](docs/CONFIGURATION.md) for detailed options.

## Dependencies

* gtk3
* libappindicator

### XIM

* libxcb
* cairo
