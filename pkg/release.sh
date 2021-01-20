#!/bin/sh

set -e

cd $(readlink -f $(dirname $0))/..

cargo build --release

mkdir -pv build/out

cp target/release/kime-xim build/out/kime-xim
cp target/release/libkime_gtk3.so build/out/im-kime.so
cp target/release/libkime_engine.so build/out/libkime_engine.so

cargo run --release -p kime-engine-config-writer > build/out/config.yaml
cp engine/cffi/kime_engine.h build/out

strip -s build/out/kime-xim
strip -s build/out/libkime_engine.so
strip -s build/out/im-kime.so

