#!/bin/sh

set -e

cd $(readlink -f $(dirname $0))/..

VER=$(grep '^version =' gtk3/Cargo.toml|head -n1|cut -d\" -f2)

mkdir -pv build/deb/kime/DEBIAN

sed "s/%VER%/${VER}/" pkg/control.in > build/deb/kime/DEBIAN/control

PREFIX=$PWD/build/deb/kime pkg/install.sh

cd build/deb

dpkg-deb --build kime

mv kime.deb "kime_${VER}_amd64.deb"

rm -rf kime

