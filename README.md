# kime

[<img src="./docs/assets/kime-roundy-default-without-text-bluegrey.png" height="100">](https://github.com/Riey/kime)

Korean IME

## View in other languages

[**English**](./README.md), [한국어](./README.ko.md)

---

[<img alt="build" src="https://img.shields.io/github/workflow/status/Riey/kime/CI?style=for-the-badge" height="25">](https://github.com/Riey/kime/actions?query=workflow%3ACI)
[<img alt="discord" src="https://img.shields.io/discord/801107569505992705.svg?style=for-the-badge" height="25">](https://discord.gg/YPnEfZqC6y)
[<img alt="release version" src="https://img.shields.io/github/v/release/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/releases)
[<img alt="aur version" src="https://img.shields.io/aur/version/kime-bin?style=for-the-badge" height="25">](https://aur.archlinux.org/packages/kime-bin/)
[<img alt="license" src="https://img.shields.io/github/license/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/blob/master/LICENSE)
[<img src="https://d1u4yishnma8v5.cloudfront.net/mobile-gift.png" alt="donaricano-btn" height="50">](https://donaricano.com/mypage/1610220543_mjZDXO)

## [Changelog](docs/CHANGELOG.md)

## Why kime

* Well tested input engine
* Very [fast](https://github.com/Riey/kime/wiki/Performance)
* Low memory footprint
* Write in mostly Rust no segfaults
* Allow custom layouts

## Have a question?

Please contact with [Discord](https://discord.gg/YPnEfZqC6y) or post issue

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

### Debian, Ubuntu

you can install from .deb file at [releases](https://github.com/Riey/kime/releases) tab.

### Build from source

#### Docker

It's convenient because you don't need install other dependencies

```sh
git clone https://github.com/riey/kime
cd kime

docker build --file build-docker/<distro path>/Dockerfile --tag kime-build:git .
docker run --name kime kime-build:git
docker cp kime:/opt/kime-out/kime.tar.xz .
# if you want deb file try this command instead
# docker cp kime:/opt/kime-out/kime_amd64.deb .
```

#### Manually build

make sure **cargo** and other dependencies listed below are installed before build.

```sh
git clone https://github.com/Riey/kime
cd kime

scripts/build.sh -ar
```

Now all files are in build/out if you want manual install go ahead

you can also use `scripts/install.sh <install-prefix>` useful script for packaging

and there is `scripts/release-deb.sh <deb-out-path>` it make `deb` file.

#### GTK

you may don't need to do this when you install with package

because most distros doing this themselves.

```sh
# If you install gtk2
sudo gtk-query-immodules-2.0 --update-cache
# If you install gtk3
sudo gtk-query-immodules-3.0 --update-cache
# If you install gtk4
sudo gio-querymodules /usr/lib/gtk-4.0/4.0.0/immodules
```

## Configuration

### Setup environment variables

#### Debian-like

Set input method `kime` in language setting

#### Other

Add the following to your init script

```sh
export GTK_IM_MODULE=kime
export QT_IM_MODULE=kime
export XMODIFIERS=@im=kime
```

if you use X it could be done in .xprofile

### Start additional servers

kime install several .desktop file that you can run

| `kime-xim` | `kime-wayland` | `kime-window` |
|------------|----------------|---------------|
| for xim    | for wayland    | for hanja, emoji, indicator |

if you use DE consider add these servers as your start application

if you doesn't have `start application` such as `i3` or `sway` that just run this commands in your config file

### Configuration

Read [CONFIGURATION.md](docs/CONFIGURATION.md) for detailed options.

## Dependencies

### Run time

Note that you only need deps what you need
for example, if you don't use qt6 it won't required.

* gtk2
* gtk3
* gtk4
* qt5
* qt6
* libappindicator-gtk3 (indicator)
* libxcb (xim)
* cairo (xim)

### Build time (you don't need when run compiled binary)

#### Required

* cmake
* cargo
* libclang
* pkg-config

#### Optional

* gtk2
* gtk3
* gtk4
* qtbase5-private
* qtbase6-private
* libappindicator-gtk3
* libxcb
* cairo
