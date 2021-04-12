#!/usr/bin/env bash

source $(dirname $0)/tool.sh

mkdir -pv $KIME_OUT

if [ -z "$KIME_MAKE_ARGS" ]; then
    KIME_MAKE_ARGS="-j4"
fi

if [ -z "$KIME_SKIP_ENGINE" ]; then
    KIME_SKIP_ENGINE=0
fi

if [ -z "$KIME_BUILD_CHECK" ]; then
    KIME_BUILD_CHECK=0
fi

if [ -z "$KIME_BUILD_XIM" ]; then
    KIME_BUILD_XIM=0
fi

if [ -z "$KIME_BUILD_WAYLAND" ]; then
    KIME_BUILD_WAYLAND=0
fi

if [ -z "$KIME_BUILD_INDICATOR" ]; then
    KIME_BUILD_INDICATOR=0
fi

set_release() {
    TARGET_DIR=./target/release
    _KIME_CARGO_ARGS="--release"
    _KIME_CMAKE_ARGS="-DCMAKE_BUILD_TYPE=Release"
}

set_debug() {
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
            echo "build.sh"
            echo "Please set KIME_BUILD_{CHECK,INDICATOR,XIM,WAYLAND}, KIME_CMAKE_ARGS or use -a"
            echo "-h: help"
            echo "-r: release mode(default)"
            echo "-d: debug mode"
            echo "-a: all modules"
            exit 0
            ;;
        r)
            set_release
            ;;
        d)
            set_debug
            ;;
        a)
            KIME_BUILD_CHECK=1
            KIME_BUILD_INDICATOR=1
            KIME_BUILD_WAYLAND=1
            if (pkg-config --exists xcb && pkg-config --exists cairo); then
                KIME_BUILD_XIM=1
            fi
            KIME_BUILD_KIME=1
            KIME_CMAKE_ARGS="-DENABLE_GTK2=ON -DENABLE_GTK3=ON -DENABLE_GTK4=ON -DENABLE_QT5=ON -DENABLE_QT6=ON $KIME_CMAKE_ARGS"
            ;;
    esac
done

echo Build rust pkgs

KIME_RUST_PKGS=()

if [ "$KIME_SKIP_ENGINE" -eq "1" ]; then
    _KIME_CMAKE_ARGS="${_KIME_CMAKE_ARGS} -DUSE_SYSTEM_ENGINE=ON"
    KIME_BUILD_CHECK=0
    echo Use system engine
else
    LD_LIBRARY_PATH="${LD_LIBRARY_PATH}:${PWD}/${TARGET_DIR}"
    KIME_RUST_PKGS+=("-pkime-engine-capi")
fi

if [ "$KIME_BUILD_CHECK" -eq "1" ]; then
    KIME_RUST_PKGS+=("-pkime-check")
fi

if [ "$KIME_BUILD_INDICATOR" -eq "1" ]; then
    KIME_RUST_PKGS+=("-pkime-indicator")
fi

if [ "$KIME_BUILD_XIM" -eq "1" ]; then
    KIME_RUST_PKGS+=("-pkime-xim")
fi

if [ "$KIME_BUILD_WAYLAND" -eq "1" ]; then
    KIME_RUST_PKGS+=("-pkime-wayland")
fi

if [ "$KIME_BUILD_KIME" -eq "1" ]; then
    KIME_RUST_PKGS+=("-pkime")
fi

cargo_build "${KIME_RUST_PKGS[@]}"

cp $TARGET_DIR/libkime_engine.so $KIME_OUT || true
cp $TARGET_DIR/kime-check $KIME_OUT || true
cp $TARGET_DIR/kime-indicator $KIME_OUT || true
cp $TARGET_DIR/kime-xim $KIME_OUT || true
cp $TARGET_DIR/kime-wayland $KIME_OUT || true
cp $TARGET_DIR/kime $KIME_OUT || true

cp src/engine/cffi/kime_engine.h $KIME_OUT
cp src/engine/cffi/kime_engine.hpp $KIME_OUT
cp docs/CHANGELOG.md $KIME_OUT
cp LICENSE $KIME_OUT
cp NOTICE.md $KIME_OUT
cp README.md $KIME_OUT
cp README.ko.md $KIME_OUT
cp -R res/* $KIME_OUT

mkdir -pv build/cmake

echo Build gtk qt immodules...

cd build/cmake

cmake ../../src $_KIME_CMAKE_ARGS $KIME_CMAKE_ARGS

make $KIME_MAKE_ARGS

cp lib/* $KIME_OUT || true

