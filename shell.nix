{
  pkgs ? import <nixpkgs> {},
}:
with pkgs;
stdenv.mkDerivation {
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
    qt5.wrapQtAppsHook
    llvmPackages_11.libclang
    llvmPackages_11.bintools
    dpkg
    gnutar
    zstd
    rustc cargo
    cmake
    extra-cmake-modules
    qt5.wrapQtAppsHook
  ];
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
}

