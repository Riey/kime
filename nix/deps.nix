{ pkgs }:
with pkgs;
{
  kimeBuildInputs = [
    dbus
    libdbusmenu

    xorg.libxcb
    libGL
    wayland
    libxkbcommon

    gtk3
    gtk4

    qt5.qtbase
    # qt6.qtbase
  ];

  kimeNativeBuildInputs = [
    python3 # xcb 0.9.0
    pkg-config
    llvmPackages_14.clang
    llvmPackages_14.libclang.lib
    llvmPackages_14.bintools
    rustc cargo
    cmake
    extra-cmake-modules
  ];
}

