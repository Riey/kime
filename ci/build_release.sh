#!/bin/bash

cargo xtask build XIM WAYLAND GTK2 GTK3 GTK4 QT5 QT6
tar cvf - -C ./build/out . | xz -9 -T0 -c - > /opt/kime-out/kime.tar.xz
cargo xtask release-deb /opt/kime-out
