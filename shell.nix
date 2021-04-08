{
  moz_overlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
  pkgs ? import <nixpkgs> { overlays = [ moz_overlay ]; },
}:
with pkgs;
let
  rust-toolchain = pkgs.rustChannelOf { version = "1.51.0"; channel = "stable"; };
in
llvmPackages_11.stdenv.mkDerivation {
  name = "kime-shell";

  dontUseCmakeConfigure = true;

  buildInputs = [
    dbus
    dbus_libs
    libdbusmenu

    xorg.libxcb
    xlibs.libpthreadstubs
    xlibs.libXdmcp.dev

    cairo
    pcre


    glib
    libselinux.dev
    libsepol.dev
    utillinux.dev
    gtk2
    gtk3
    at_spi2_core.dev
    epoxy.dev
    xlibs.libXtst

    # gtk4

    qt5.qtbase
    # qt6.qtbase
  ];
  nativeBuildInputs = [
    bash
    pkg-config
    clang_11
    llvmPackages_11.libclang
    llvmPackages_11.bintools
    dpkg
    gnutar
    zstd
    cmake
    extra-cmake-modules
    qt5.wrapQtAppsHook

    rust-toolchain.rust
  ];
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
}

