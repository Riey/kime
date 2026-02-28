#!/bin/bash
set -e

MULTIARCH=$(dpkg-architecture -qDEB_HOST_MULTIARCH)
meson setup build --prefix=/usr --libdir="lib/$MULTIARCH" $MESON_ARGS
ninja -C build

for f in kime kime-check kime-indicator kime-candidate-window kime-xim kime-wayland; do
    [ -f "target/release/$f" ] && strip -s "target/release/$f"
done
strip -s target/release/libkime_engine.so
for f in build/src/frontends/*/lib*.so; do
    [ -e "$f" ] && strip -s "$f"
done

packaging/build-deb.sh build /opt/kime-out
