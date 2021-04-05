{
  moz_overlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
  nixpkgs ? <nixpkgs>,
  pkgs ? import nixpkgs { overlays = [ moz_overlay ]; },
}:
with pkgs;
llvmPackages_11.stdenv.mkDerivation {
  name = "kime-shell";
  buildInputs = [
    dbus
    dbus_libs
    libdbusmenu
    glib
    xorg.libxcb
    pcre
    cairo

    gtk2
    gtk3
    # gtk4

    qt5.qtbase
    # qt6.qtbase
  ];
  nativeBuildInputs = [
    pkgconfig
    clang_11
    llvmPackages_11.libclang
    llvmPackages_11.bintools
    pkgs.latest.rustChannels.stable.rust
    cmake
    extra-cmake-modules
  ];
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
  CC = "${clang_11}/bin/clang";
  CXX = "${clang_11}/bin/clang++";
}

