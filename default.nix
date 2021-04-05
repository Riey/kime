{
  moz_overlay ? import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz),
  pkgs ? import <nixpkgs> { overlays = [ moz_overlay ]; },
  gis ? import (fetchTarball {
    url = https://github.com/icetan/nix-git-ignore-source/archive/v1.0.0.tar.gz;
    sha256 = "1mnpab6x0bnshpp0acddylpa3dslhzd2m1kk3n0k23jqf9ddz57k";
  }) {},
}:
with pkgs;
let
  kimeVersion = "2.3.2";
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
    llvmPackages_11.libclang
    llvmPackages_11.bintools
    pkgs.latest.rustChannels.stable.rust
    git
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
  cargoSha256 = "19jya8zwh5k0ldzq51n9xxm2baalmjif6w87g16p071mk5h5p0hp";
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

