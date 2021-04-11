{ pkgs }:
with pkgs;
{
  kimeBuildInputs = [
    dbus
    dbus_libs
    libdbusmenu

    xorg.libxcb
    xlibs.libpthreadstubs
    xlibs.libXdmcp.dev

    cairo
    pcre

    glib
    libthai
    libdatrie
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
  kimeNativeBuildInputs = [
    bash
    pkg-config
    clang_11
    llvmPackages_11.libclang
    llvmPackages_11.bintools
    rustc cargo
    cmake
    extra-cmake-modules
  ];
}

