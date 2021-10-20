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
  nativeBuildInputs = deps.kimeNativeBuildInputs;
  CMAKE_EXPORT_COMPILE_COMMANDS = 1;
  LIBCLANG_PATH = "${pkgs.llvmPackages_13.libclang.lib}/lib";
  LD_LIBRARY_PATH = "${pkgs.wayland}/lib:${pkgs.libGL}/lib";
  RUST_BACKTRACE = 1;
}

