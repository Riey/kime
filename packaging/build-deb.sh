#!/bin/bash
set -e
# Usage: build-deb.sh <meson-build-dir> <output-dir>
BUILD_DIR=${1:?}
OUT_DIR=${2:?}
VER=$(cat VERSION)
ARCH=$(dpkg --print-architecture)
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$TMP/DEBIAN"
sed "s/%VER%/$VER/; s/%ARCH%/$ARCH/" packaging/control.in > "$TMP/DEBIAN/control"

mkdir -p "$TMP/usr/share/im-config/data"
cp packaging/im_kime.conf "$TMP/usr/share/im-config/data/51_kime.conf"
cp packaging/im_kime.rc "$TMP/usr/share/im-config/data/51_kime.rc"

DESTDIR="$TMP" meson install -C "$BUILD_DIR" --no-rebuild

dpkg-deb --root-owner-group --build "$TMP" "$OUT_DIR/kime_${VER}_${ARCH}.deb"
