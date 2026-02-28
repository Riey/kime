#!/bin/bash
set -e

meson setup build $MESON_ARGS
ninja -C build

for f in kime kime-check kime-indicator kime-candidate-window kime-xim kime-wayland; do
    [ -f "target/release/$f" ] && strip -s "target/release/$f"
done
strip -s target/release/libkime_engine.so
strip -s build/src/frontends/*/lib*.so

DESTDIR=$(pwd)/kime-install ninja -C build install
tar -cvf - -C kime-install . | zstd -T0 -15 -o /opt/kime-out/kime.tar.zst
