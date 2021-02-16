#!/bin/bash

source $(dirname $0)/tool.sh

mkdir -pv $KIME_OUT

if [ -z "$KIME_MAKE_ARGS" ]; then
    KIME_MAKE_ARGS="-j4"
fi

set_release() {
    NEED_STRIP=1
    TARGET_DIR=./target/release
    _KIME_CARGO_ARGS="--release"
    _KIME_CMAKE_ARGS="-DCMAKE_BUILD_TYPE=Release"
}

set_debug() {
    NEED_STRIP=0
    TARGET_DIR=./target/debug
    _KIME_CARGO_ARGS=""
    _KIME_CMAKE_ARGS="-DCMAKE_BUILD_TYPE=Debug"
}

cargo_build() {
    cargo build $_KIME_CARGO_ARGS $KIME_CARGO_ARGS "$@"
}

set_release

while getopts hrda opt; do
    case $opt in
        h)
            echo "-r: release mode(default)"
            echo "-d: debug mode"
            echo "-a: all immodules"
            exit 0
            ;;
        r)
            set_release
            ;;
        d)
            set_debug
            ;;
        a)
            KIME_CMAKE_ARGS="-DENABLE_GTK2=ON -DENABLE_GTK3=ON -DENABLE_GTK4=ON -DENABLE_QT5=ON -DENABLE_QT6=ON $KIME_CMAKE_ARGS"
            ;;
    esac
done

LD_LIBRARY_PATH=$LD_LIBRARY_PATH:$PWD/$TARGET_DIR

echo Build core...

cargo_build -p kime-engine-capi

echo Build xim wayland indicator check...

cargo_build -p kime-check -p kime-xim -p kime-wayland -p kime-indicator

cp $TARGET_DIR/kime-check $KIME_OUT
cp $TARGET_DIR/kime-xim $KIME_OUT
cp $TARGET_DIR/kime-wayland $KIME_OUT
cp $TARGET_DIR/kime-indicator $KIME_OUT
cp src/engine/cffi/kime_engine.h $KIME_OUT
cp src/engine/cffi/kime_engine.hpp $KIME_OUT
cp $TARGET_DIR/libkime_engine.so $KIME_OUT
cp LICENSE $KIME_OUT
cp res/default_config.yaml $KIME_OUT
cp -R res/icons $KIME_OUT

mkdir -pv build/cmake

echo Build gtk qt immodules...

cd build/cmake

cmake ../../src $_KIME_CMAKE_ARGS $KIME_CMAKE_ARGS

make $KIME_MAKE_ARGS

cp lib/* $KIME_OUT

if [ $NEED_STRIP -eq "1" ]; then
    strip -s $KIME_OUT/* 2&>/dev/null || true
fi
