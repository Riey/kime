#!/bin/bash

KIME_PREFIX=51_kime

source $(dirname $0)/tool.sh

if [ -z "$1" ]; then
    echo "Usage: <release-deb.sh> <out-path>"
    exit 1
fi

TARGET_PATH=$1
TMP_PATH=$(mktemp -d)
VER=$(git tag --sort=v:refname | tail -1 | cut -b2-)

mkdir -pv $TMP_PATH/DEBIAN
mkdir -pv $TMP_PATH/usr/share/im-config/data

cat scripts/control.in | sed "s/%VER%/$VER/" > $TMP_PATH/DEBIAN/control
cp scripts/im_kime.rc $TMP_PATH/usr/share/im-config/data/$KIME_PREFIX.rc
cp scripts/im_kime.conf $TMP_PATH/usr/share/im-config/data/$KIME_PREFIX.conf

KIME_INSTALL_HEADER=0 \
KIME_LIB_DIR=usr/lib/x86_64-linux-gnu \
KIME_QT5_DIR=usr/lib/x86_64-linux-gnu/qt5 \
KIME_QT6_DIR=usr/lib/x86_64-linux-gnu/qt6 \
scripts/install.sh $TMP_PATH

dpkg-deb --build $TMP_PATH "${TARGET_PATH}/kime_amd64.deb"
