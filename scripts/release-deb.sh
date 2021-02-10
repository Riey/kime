#!/bin/bash

source $(dirname $0)/tool.sh

if [ -z "$1" ]; then
    echo "Usage: <install.sh> <install-prefix>"
    exit 1
fi

TARGET_PATH=$1
TMP_PATH=$(mktemp -d)
VER=$(git tag --sort=v:refname | tail -1 | cut -b2-)

mkdir -pv $TMP_PATH/DEBIAN

cat scripts/control.in | sed "s/%VER%/$VER/" > $TMP_PATH/DEBIAN/control

scripts/install.sh $TMP_PATH

dpkg-deb --build $TMP_PATH "${TARGET_PATH}/kime_${VER}_amd64.deb"

