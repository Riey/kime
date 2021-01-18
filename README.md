# kime

Korean IME

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
./pkg/release-7z.sh
7z x -obuild/out build/7z/kime.7z

cd build/out

sudo install -Dm755 kime-xim -t "/usr/bin"
sudo install -Dm755 im-kime.so -t "/usr/lib/gtk-3.0/3.0.0/immodules"
sudo install -Dm755 libkime_engine.so -t "/usr/lib"
sudo install -Dm644 kime_engine.h -t "/usr/include/kime"
sudo install -Dm644 config.yaml -t "/etc/kime"
```

## Configuration
add the folowing to .xprofile or .xinitrc and restart X:

```sh
export GTK_IM_MODULE=kime
kime-xim &
export XMODIFIERS=@im=kime
```

read (CONFIGURATION.md) for detailed options.

## Dependencies

### GTK3

* gtk3
* pango

### XIM

* libxcb
* cairo
