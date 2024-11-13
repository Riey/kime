# kime

[<img src="./docs/assets/kime-roundy-default-without-text-bluegrey.png" height="100">](https://github.com/Riey/kime)

Korean IME

## View in other languages

[**English**](./README.md), [한국어](./README.ko.md)

---

[<img alt="build" src="https://img.shields.io/github/actions/workflow/status/Riey/kime/ci.yaml?style=for-the-badge&branch=develop" height="25">](https://github.com/Riey/kime/actions?query=workflow%3ACI)
[<img alt="discord" src="https://img.shields.io/discord/801107569505992705.svg?style=for-the-badge" height="25">](https://discord.gg/YPnEfZqC6y)
[<img alt="release version" src="https://img.shields.io/github/v/release/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/releases)
[<img alt="aur version" src="https://img.shields.io/aur/version/kime-bin?style=for-the-badge" height="25">](https://aur.archlinux.org/packages/kime-bin/)
[<img alt="license" src="https://img.shields.io/github/license/Riey/kime?style=for-the-badge" height="25">](https://github.com/Riey/kime/blob/master/LICENSE)

## [Changelog](docs/CHANGELOG.md)

## Why kime

* Well tested input engine
* Blazing [fast](https://github.com/Riey/kime/wiki/Performance)
* Small memory footprint
* Written in Rust, no segmentation fault
* Custom layouts

## Have a question?

Please contact us on [Discord](https://discord.gg/YPnEfZqC6y) or create github issue.

## Supported frontend

- [x] XIM
- [x] Wayland
- [x] GTK3
- [x] GTK4
- [x] Qt5
- [x] Qt6

## Installation

### NixOS

Add this code to your configuration.nix

```nix
i18n = {
  defaultLocale = "en_US.UTF-8";
  inputMethod = {
    enable = true;
    type = "kime";
    kime.config = {
      indicator.icon_color = "White";
    };
  };
};
```

### Arch Linux

Latest release of kime is available on [`kime` AUR package](https://aur.archlinux.org/packages/kime).

Developing version is available on [`kime-git`](https://aur.archlinux.org/packages/kime-git).

### Debian, Ubuntu

`.deb` package is available on github [releases](https://github.com/Riey/kime/releases) tab.

### Fedora

The unofficial package is being maintained on [Fedora Copr](https://copr.fedorainfracloud.org/coprs/toroidalfox/kime/).

```sh
dnf copr enable toroidalfox/kime
dnf install kime # `kime-git` for bleeding edge
```

### Gentoo

```sh
eselect repository add riey git https://github.com/Riey/overlay
eselect repository enable riey
emaint sync -r riey
emerge -av kime
```

### Build from source

#### Docker

Building with docker does not requires any other dependencies.

```sh
git clone https://github.com/riey/kime
cd kime

docker build --file build-docker/<distro path>/Dockerfile --tag kime-build:git .
docker run --name kime kime-build:git
docker cp kime:/opt/kime-out/kime.tar.xz .
# if you want to build deb package try this command instead
# docker cp kime:/opt/kime-out/kime_amd64.deb .
```

#### Manual build

Make sure that **cargo** and other dependencies listed below are installed before build.

```sh
git clone https://github.com/Riey/kime
cd kime

scripts/build.sh -ar
```

Every files needed for manual install is in `build/out` directory.

`scripts/install.sh <install-prefix>` can be used for packaging.

`scripts/release-deb.sh <deb-out-path>` can be used for packaging `deb` package.

#### GTK

Typically, this step is not necessary when kime is installed from binary package because most Linux distros does these steps themselves.

```sh
# for gtk3
sudo gtk-query-immodules-3.0 --update-cache
# for gtk4
sudo gio-querymodules /usr/lib/gtk-4.0/4.0.0/immodules
```

## Development

### C/C++

Run `./scripts/generate_properties.sh` for using intellisense C/C++ in vscode

## Configuration

### environment variables setup

#### Debian-like

Set input method as `kime` in language setting

#### Others

Append following lines to your init script

```sh
export GTK_IM_MODULE=kime
export QT_IM_MODULE=kime
export XMODIFIERS=@im=kime
```

if you use X, append above lines to file `~/.xprofile`

### Start additional server

kime.desktop file is installed in /etc/xdg/autostart when installing kime.

### KDE Plasma Wayland

It is required to select `kime daemon` under System Settings > Hardware > Input Devices > Virtual Keyboard.  
A logout is recommended afterwards.

### Weston

It is required to have the following lines in `~/.config/weston.ini`
```
[input-method]
path=/usr/bin/kime
```

### Configuration

Read [CONFIGURATION.md](docs/CONFIGURATION.md) for detail options.

## Dependencies

### Run time

These dependencies are optional depending on your environments. For example, qt6 is not required when it is not used.

* gtk3
* gtk4
* qt5
* qt6
* libdbus (indicator)
* xcb (candidate)
* fontconfig (xim)
* freetype (xim)
* libxkbcommon (wayland)

### Build time (you don't need this on running compiled binary)

#### Required

* cmake
* cargo
* libclang
* pkg-config

#### Optional

* gtk3
* gtk4
* qtbase5-private
* qtbase6-private
* libdbus
* xcb
* fontconfig
* freetype
* libxkbcommon
