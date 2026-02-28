{
  pkgs ? import <nixpkgs> {},
  rustToolchain ? pkgs.rustc,
  gtk3 ? true,
  gtk4 ? true,
  qt5 ? false,
  qt6 ? true,
}:
let
  deps = import ./nix/deps.nix { inherit pkgs gtk3 gtk4 qt5 qt6; };
  stdenv = pkgs.llvmPackages_18.stdenv;
  mkShell = (pkgs.mkShell.override { inherit stdenv; });
in
mkShell {
  name = "kime-shell";
  dontWrapQtApps = true;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ [
    rustToolchain
    pkgs.gedit
    pkgs.llvmPackages_18.lldb
  ];
  LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";
  LD_LIBRARY_PATH = "./target/debug:${pkgs.wayland}/lib:${pkgs.libGL}/lib:${pkgs.libxkbcommon}/lib";
  G_MESSAGES_DEBUG = "kime";
  GTK_IM_MODULE = "kime";
  GTK_IM_MODULE_FILE = builtins.toString ./.vscode/immodules.cache;
  RUST_BACKTRACE = 1;
}

