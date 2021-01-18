#!/bin/sh

set -e

cargo build --release

mkdir -pv build/7z

cp target/release/kime-xim build/7z/kime-xim
cp target/release/libkime_gtk3.so build/7z/im-kime.so
cp target/release/libkime_engine.so build/7z/libkime_engine.so

cp engine/capi/data/config.yaml build/7z
cp engine/cffi/kime_engine.h build/7z

strip -s build/7z/kime-xim
strip -s build/7z/libkime_engine.so
strip -s build/7z/im-kime.so

7z a build/kime.7z ./build/7z/*

rm build/7z/*

mv build/kime.7z build/7z
