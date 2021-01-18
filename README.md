# kime

Korean IME

[![ci](https://github.com/Riey/kime/workflows/CI/badge.svg)](https://github.com/Riey/kime/actions?query=workflow%3ACI)
[![release](https://github.com/Riey/kime/workflows/Release/badge.svg)](https://github.com/Riey/kime/actions?query=workflow%3ARelease)
[![Release version](https://img.shields.io/github/v/release/Riey/kime)](https://github.com/Riey/kime/releases)
[![AUR version](https://img.shields.io/aur/version/kime)](https://aur.archlinux.org/packages/kime/)
[![LICENSE](https://img.shields.io/github/license/Riey/kime)](https://github.com/Riey/kime/blob/master/LICENSE)

## Why kime

* Well tested input engine
* Low memory footprint
* Write in Rust no segfaults
* Allow custom layouts

## Supported frontend

- [x] XIM
- [ ] Wayland
- [ ] GTK2
- [x] GTK3
- [ ] GTK4
- [ ] Qt4
- [ ] Qt5

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
export XMODIFIERS=@im=kime
kime-xim &
```

read [CONFIGURATION.md](CONFIGURATION.md) for detailed options.

## Dependencies

### GTK3

* gtk3
* pango

### XIM

* libxcb
* cairo
