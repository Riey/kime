#!/bin/sh

set -e

cd $(readlink -f $(dirname $0))/../build/out

install -Dm755 kime-xim -t "$PREFIX/usr/bin"
install -Dm755 im-kime.so -t "$PREFIX/usr/lib/gtk-3.0/3.0.0/immodules"
install -Dm755 libkime_engine.so -t "$PREFIX/usr/lib"
install -Dm644 kime_engine.h -t "$PREFIX/usr/include/kime"
install -Dm644 config.yaml -t "$PREFIX/etc/kime"

