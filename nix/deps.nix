{ pkgs, gtk3 ? true, gtk4 ? true, qt5 ? false, qt6 ? true }:
{
  kimeBuildInputs = with pkgs; [
    dbus
    libdbusmenu

    xorg.libxcb
    libGL
    wayland
    libxkbcommon
  ]
  ++ pkgs.lib.optional gtk3 pkgs.gtk3
  ++ pkgs.lib.optional gtk4 pkgs.gtk4
  ++ pkgs.lib.optional qt5 pkgs.qt5.qtbase
  ++ pkgs.lib.optional qt6 pkgs.qt6.qtbase;

  kimeNativeBuildInputs = with pkgs; [
    python3 # xcb 0.9.0
    pkg-config
    llvmPackages_18.clang
    llvmPackages_18.libclang.lib
    llvmPackages_18.bintools
    meson
    ninja
  ];
}

