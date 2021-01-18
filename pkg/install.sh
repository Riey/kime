#!/bin/sh

set -e

cd $(dirname "$0")/../build/out

install -Dm755 kime-xim -t "/usr/bin"
install -Dm755 im-kime.so -t "/usr/lib/gtk-3.0/3.0.0/immodules"
install -Dm755 libkime_engine.so -t "/usr/lib"
install -Dm644 kime_engine.h -t "/usr/include/kime"
install -Dm644 config.yaml -t "/etc/kime"

