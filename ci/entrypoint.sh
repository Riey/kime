#!/bin/bash

set -e

if [ "$1" == "ci" ]; then
    echo "RUN CI MODE"
    cargo xtask build --mode Debug XIM WAYLAND GTK2 GTK3 GTK4 QT5 QT6
    cargo xtask test
elif [ "$1" == "release" ]; then
    echo "RUN RELEASE MODE"
    cargo xtask build XIM WAYLAND GTK2 GTK3 GTK4 QT5 QT6
    7z a /opt/kime-out/kime.7z ./build/out/*
    cargo xtask release-deb /opt/kime-out
fi
