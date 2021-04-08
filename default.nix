{
  pkgs ? import <nixpkgs> {},
  gis ? import (fetchTarball {
    url = https://github.com/icetan/nix-git-ignore-source/archive/v1.0.0.tar.gz;
    sha256 = "1mnpab6x0bnshpp0acddylpa3dslhzd2m1kk3n0k23jqf9ddz57k";
  }) {},
}:
with pkgs;
let
  kimeVersion = builtins.readFile ./VERSION;
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
    qt5.wrapQtAppsHook
    llvmPackages_11.libclang
    llvmPackages_11.bintools
    rustc cargo
    cmake
    extra-cmake-modules
  ];
in
rustPlatform.buildRustPackage rec {
  src = gis.gitIgnoreSource ./.;
  pname = "kime";
  buildInputs = kimeBuildInputs;
  nativeBuildInputs = kimeNativeBuildInputs;
  version = kimeVersion;
  cargoSha256 = "146m1kg83bmlf6sxi20yawksp47qp0byrh07wkv56vyl5p02fsz1";
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
  dontUseCmakeConfigure = true;
  buildPhase = "bash scripts/build.sh -ar";
  installPhase = ''
    KIME_BIN_DIR=bin \
    KIME_INSTALL_HEADER=1 \
    KIME_INCLUDE_DIR=include \
    KIME_ICON_DIR=share/icons \
    KIME_LIB_DIR=lib \
    KIME_QT5_DIR=lib/qt-${pkgs.qt5.qtbase.version} \
    bash scripts/install.sh "$out"
  '';
}

