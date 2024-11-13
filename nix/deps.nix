{ pkgs }:
{
  kimeBuildInputs = with pkgs; [
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

  kimeNativeBuildInputs = with pkgs; [
    python3 # xcb 0.9.0
    pkg-config
    llvmPackages_18.clang
    llvmPackages_18.libclang.lib
    llvmPackages_18.bintools
    rustc cargo
    cmake
    extra-cmake-modules
  ];
}

