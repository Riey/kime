{
  pkgs ? import <nixpkgs> { },
}:
let
  deps = import ./nix/deps.nix { inherit pkgs; };
  stdenv = pkgs.llvmPackages_18.stdenv;
  mkShell = (pkgs.mkShell.override { inherit stdenv; });
in
pkgs.mkShell {
  name = "kime-shell";
  dontUseCmakeConfigure = true;
  dontWrapQtApps = true;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ (with pkgs; [
    rustfmt
    pkgs.gedit
    llvmPackages_18.lldb
  ]);
  CMAKE_EXPORT_COMPILE_COMMANDS = 1;
  LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";
  LD_LIBRARY_PATH = "./target/debug:${pkgs.wayland}/lib:${pkgs.libGL}/lib:${pkgs.libxkbcommon}/lib";
  G_MESSAGES_DEBUG = "kime";
  GTK_IM_MODULE = "kime";
  GTK_IM_MODULE_FILE = builtins.toString ./.vscode/immodules.cache;
  RUST_BACKTRACE = 1;
}

