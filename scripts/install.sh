#!/bin/bash

source $(dirname $0)/tool.sh

if [ -z "$1" ]; then
    echo "Usage: <install.sh> <install-prefix>"
    exit 1
fi

if [ -z "$KIME_INSTALL_HEADER" ]; then
    KIME_INSTALL_HEADER=0
fi

PREFIX=$1

if [ -z "$KIME_BIN_DIR" ]; then
    KIME_BIN_DIR=usr/bin
fi

if [ -z "$KIME_INCLUDE_DIR" ]; then
    KIME_INCLUDE_DIR=usr/include
fi

if [ -z "$KIME_CONFIG_DIR" ]; then
    KIME_CONFIG_DIR=etc/xdg/kime
fi

if [ -z "$KIME_ICON_DIR" ]; then
    KIME_ICON_DIR=usr/share/icons
fi

if [ -z "$KIME_AUTOSTART_DIR" ]; then
    KIME_AUTOSTART_DIR=etc/xdg/autostart
fi

if [ -z "$KIME_LIB_DIR" ]; then
    KIME_LIB_DIR=usr/lib
fi

if [ -z "$KIME_GTK2_DIR" ]; then
    KIME_GTK2_DIR="$KIME_LIB_DIR/gtk-2.0/2.10.0/immodules"
fi

if [ -z "$KIME_GTK3_DIR" ]; then
    KIME_GTK3_DIR="$KIME_LIB_DIR/gtk-3.0/3.0.0/immodules"
fi

if [ -z "$KIME_GTK4_DIR" ]; then
    KIME_GTK4_DIR="$KIME_LIB_DIR/gtk-4.0/4.0.0/immodules"
fi

if [ -z "$KIME_QT5_DIR" ]; then
    KIME_QT5_DIR="$KIME_LIB_DIR/qt"
fi

if [ -z "$KIME_QT6_DIR" ]; then
    KIME_QT6_DIR="$KIME_LIB_DIR/qt6"
fi

install_if () {
    if [ -f $KIME_OUT/$1 ]; then
        install -Dm$2 $KIME_OUT/$1 $3 "$PREFIX/$4"
    else
        echo "SKIP $1"
    fi
}

install_bin () {
    install_if $1 755 -t "$KIME_BIN_DIR"
}

install_bin kime-check
install_bin kime-indicator
install_bin kime-xim
install_bin kime-wayland

if [ $KIME_INSTALL_HEADER -eq "1" ]; then
    install -Dm644 $KIME_OUT/kime_engine.h -t "$PREFIX/$KIME_INCLUDE_DIR"
    install -Dm644 $KIME_OUT/kime_engine.hpp -t "$PREFIX/$KIME_INCLUDE_DIR"
fi

install -Dm644 $KIME_OUT/default_config.yaml -T "$PREFIX/$KIME_CONFIG_DIR/config.yaml"
install -Dm644 $KIME_OUT/*.desktop -t "$PREFIX/$KIME_AUTOSTART_DIR"
install -Dm644 $KIME_OUT/icons/64x64/*.png -t "$PREFIX/$KIME_ICON_DIR/hicolor/64x64/apps"
install -Dm755 $KIME_OUT/libkime_engine.so -t "$PREFIX/$KIME_LIB_DIR"

install_if libkime-gtk2.so 755 -T "$KIME_GTK2_DIR/im-kime.so"
install_if libkime-gtk3.so 755 -T "$KIME_GTK3_DIR/im-kime.so"
install_if libkime-gtk4.so 755 -t "$KIME_GTK4_DIR"
install_if libkime-qt5.so 755 -T "$KIME_QT5_DIR/plugins/platforminputcontexts/libkimeplatforminputcontextplugin.so"
install_if libkime-qt6.so 755 -T "$KIME_QT6_DIR/plugins/platforminputcontexts/libkimeplatforminputcontextplugin.so"
