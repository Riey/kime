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

  # Libraries the e2e clients (tests/e2e/clients) link against on top of the
  # kime build inputs: Xlib for the XIM client, XTEST for the raw-keycode
  # injector. wayland-client/xkbcommon already come from kimeBuildInputs.
  kimeE2eBuildInputs = with pkgs; [
    libx11
    libxtst
  ];

  # Runtime tooling for the headless e2e GUI suite (tests/e2e). Only the
  # devshell pulls these in — the package build never runs the suite.
  #
  # The pygobject-enabled python3 shadows the plain `python3` above, so
  # shell.nix must put this list first on PATH.
  kimeE2eNativeBuildInputs = with pkgs; [
    cargo-nextest # parallel per-process test runner (run.sh, e2e CI job)
    (python3.withPackages (ps: [ ps.pygobject3 ])) # clients/gtk_probe.py
    gobject-introspection # Gtk-3.0/4.0 typelibs -> GI_TYPELIB_PATH
    # unwrapped: the wrapper execs sway under dbus-run-session, which has no
    # session.conf to read outside NixOS — and a headless test compositor
    # needs no session bus.
    sway-unwrapped # headless compositor for the wayland tests
    xvfb # headless X server for the gtk/qt/xim tests
    xdotool # XTEST typing + window focus
    xprop # kime-xim readiness poll (XIM_SERVERS)
  ];

  # The fuzz crate (fuzz/) — nightly-only, entered via `nix develop .#fuzz`.
  kimeFuzzBuildInputs = with pkgs; [
    libhangul # engine_diff_libhangul differential target
  ];

  kimeFuzzNativeBuildInputs = with pkgs; [
    cargo-fuzz
    pkg-config
  ];
}

