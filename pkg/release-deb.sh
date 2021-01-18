#!/bin/sh

set -e

cargo build --release

mkdir -pv build/deb

cargo deb -p kime-engine-capi
cargo deb -p kime-gtk3
cargo deb -p kime-xim

cp target/debian/* build/deb
