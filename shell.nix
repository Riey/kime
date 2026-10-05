{
  pkgs ? import <nixpkgs> { },
  rustToolchain ? pkgs.symlinkJoin {
    name = "rustc-cargo";
    paths = [
      pkgs.rustc
      pkgs.cargo
    ];
  },
  gtk3 ? true,
  gtk4 ? true,
  qt5 ? false,
  qt6 ? true,
}:
let
  deps = import ./nix/deps.nix {
    inherit
      pkgs
      gtk3
      gtk4
      qt5
      qt6
      ;
  };
  inherit (deps) llvmPackages;
  stdenv = llvmPackages.stdenv;
  mkShell = (pkgs.mkShell.override { inherit stdenv; });
in
mkShell {
  name = "kime-shell";
  dontWrapQtApps = true;
  buildInputs = deps.kimeBuildInputs ++ deps.kimeE2eBuildInputs;
  # e2e tools first: their python3 carries pygobject and must win on PATH
  # over the plain python3 in kimeNativeBuildInputs.
  nativeBuildInputs =
    deps.kimeE2eNativeBuildInputs
    ++ deps.kimeNativeBuildInputs
    ++ [
      rustToolchain
      pkgs.gedit
      llvmPackages.lldb
      pkgs.nixfmt
    ];
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
  LD_LIBRARY_PATH = "./target/debug:${pkgs.wayland}/lib:${pkgs.libGL}/lib:${pkgs.libxkbcommon}/lib:${pkgs.mesa}/lib";
  # Software GL (llvmpipe) for the eframe candidate window on Xvfb, and a
  # font config, since the store has neither on a search path apps can guess.
  # D2Coding is the fallback family the engine queries for the candidate list
  # (src/engine/core/src/config.rs); with no match the window gets an empty
  # font and dies, so it is a runtime dependency, not a nicety.
  LIBGL_DRIVERS_PATH = "${pkgs.mesa}/lib/dri";
  FONTCONFIG_FILE = pkgs.makeFontsConf {
    fontDirectories = [
      pkgs.dejavu_fonts
      pkgs.d2coding
    ];
  };
  # The e2e suite clears the environment of every process it spawns
  # (tests/e2e/src/envs.rs); these are the store paths its GUI probes cannot
  # rediscover on their own.
  KIME_E2E_PASS_ENV = "LD_LIBRARY_PATH,GI_TYPELIB_PATH,LIBGL_DRIVERS_PATH,FONTCONFIG_FILE,XDG_DATA_DIRS";
  G_MESSAGES_DEBUG = "kime";
  GTK_IM_MODULE = "kime";
  GTK_IM_MODULE_FILE = builtins.toString ./.vscode/immodules.cache;
  RUST_BACKTRACE = 1;
}
