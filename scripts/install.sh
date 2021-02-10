#!/bin/bash

source $(dirname $0)/tool.sh

if [ -z "$1" ]; then
    echo "Usage: <install.sh> <install-prefix>"
    exit 1
fi

PREFIX=$1

if [ -z "$KIME_BIN_DIR" ]; then
    KIME_BIN_DIR=usr/bin
fi

if [ -z "$KIME_LIB_DIR" ]; then
    KIME_LIB_DIR=usr/lib
fi

if [ -z "$KIME_GTK2_DIR" ]; then
    KIME_GTK2_DIR="usr/lib/gtk-2.0/2.10.0/immodules"
fi

if [ -z "$KIME_GTK3_DIR" ]; then
    KIME_GTK3_DIR="usr/lib/gtk-3.0/3.0.0/immodules"
fi

if [ -z "$KIME_GTK4_DIR" ]; then
    KIME_GTK4_DIR="usr/lib/gtk-4.0/4.0.0/immodules"
fi

if [ -z "$KIME_QT5_DIR" ]; then
    KIME_QT5_DIR="usr/lib/qt"
fi

if [ -z "$KIME_QT6_DIR" ]; then
    KIME_QT6_DIR="usr/lib/qt6"
fi

install_bin() {
    install -Dm755 $KIME_OUT/$1 -t "$PREFIX/$KIME_BIN_DIR"
}

install_bin kime-indicator
install_bin kime-xim
install_bin kime-wayland

install -Dm755 $KIME_OUT/libkime_engine.so -t "$PREFIX/$KIME_LIB_DIR"
install -Dm755 $KIME_OUT/libkime-gtk2.so -T "$PREFIX/$KIME_GTK2_DIR/im-kime.so"
install -Dm755 $KIME_OUT/libkime-gtk3.so -T "$PREFIX/$KIME_GTK3_DIR/im-kime.so"
install -Dm755 $KIME_OUT/libkime-gtk4.so -t "$PREFIX/$KIME_GTK4_DIR"
install -Dm755 $KIME_OUT/libkime-qt5.so -T "$PREFIX/$KIME_QT5_DIR/plugins/platforminputcontexts/libkimeplatforminputcontextplugin.so"
install -Dm755 $KIME_OUT/libkime-qt6.so -T "$PREFIX/$KIME_QT6_DIR/plugins/platforminputcontexts/libkimeplatforminputcontextplugin.so"

