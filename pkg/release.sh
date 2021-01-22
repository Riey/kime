#!/bin/sh

set -e

cd $(readlink -f $(dirname $0))/..

# Start Build

cargo build --release

# Start Build qt5

mkdir -pv build/qt5
cd build/qt5
cmake ../../qt5 -DCMAKE_BUILD_TYPE=Release
make -j8
cd ../..

mkdir -pv build/out

# Collect files

cp target/release/kime-xim build/out/kime-xim
cp target/release/libkime_gtk3.so build/out/im-kime.so
cp target/release/libkime_engine.so build/out/libkime_engine.so
cp build/qt5/libkime-qt5.so build/out/libkimeplatforminputcontextplugin.so

target/release/kime-engine-config-writer > build/out/config.yaml
cp engine/cffi/kime_engine.h build/out

# strip binaries

strip -s build/out/kime-xim
strip -s build/out/libkime_engine.so
strip -s build/out/im-kime.so
strip -s build/out/libkimeplatforminputcontextplugin.so
