#!/bin/bash

cargo xtask build XIM WAYLAND GTK2 GTK3 GTK4 QT5 QT6
7z a /opt/kime-out/kime.7z ./build/out/*
cargo xtask release-deb /opt/kime-out
