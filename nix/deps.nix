{ pkgs }:
with pkgs;
{
  kimeBuildInputs = [
    dbus
    dbus_libs
    libdbusmenu

    xlibs.libxcb
    libGL
    wayland
    libxkbcommon

    gtk2
    gtk3
    gtk4

    qt5.qtbase
    # qt6.qtbase
  ];

  kimeNativeBuildInputs = [
    python3 # xcb 0.9.0
    pkg-config
    llvmPackages_13.clang
    llvmPackages_13.libclang.lib
    llvmPackages_13.bintools
    rustc cargo
    cmake
    extra-cmake-modules
  ];
}

