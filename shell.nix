{
  pkgs ? import <nixpkgs> { },
}:
let
  deps = import ./nix/deps.nix { pkgs = pkgs; };
in
with pkgs;
llvmPackages_13.stdenv.mkDerivation {
  name = "kime-shell";
  dontUseCmakeConfigure = true;
  dontWrapQtApps = true;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ (with pkgs; [
    gnome.gedit
    llvmPackages_13.lldb
  ]);
  CMAKE_EXPORT_COMPILE_COMMANDS = 1;
  LIBCLANG_PATH = "${pkgs.llvmPackages_13.libclang.lib}/lib";
  LD_LIBRARY_PATH = "./target/debug:${pkgs.wayland}/lib:${pkgs.libGL}/lib";
  G_MESSAGES_DEBUG = "kime";
  GTK_IM_MODULE = "kime";
  GTK_IM_MODULE_FILE = builtins.toString ./.vscode/immodules.cache;
  RUST_BACKTRACE = 1;
}

