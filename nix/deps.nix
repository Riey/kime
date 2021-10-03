{ pkgs }:
with pkgs;
{
  kimeBuildInputs = [
    dbus
    dbus_libs
    libdbusmenu

    xorg.libxcb

    cairo
    pcre

    gtk2
    gtk3

    # gtk4

    qt5.qtbase
    # qt6.qtbase
  ];
  kimeNativeBuildInputs = [
    bash
    pkg-config
    llvmPackages_13.clang
    llvmPackages_13.libclang.lib
    llvmPackages_13.bintools
    rustc cargo
    cmake
    extra-cmake-modules
  ];
}

