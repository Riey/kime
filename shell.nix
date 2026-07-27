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
    pkgs.mold
    pkgs.gedit
    pkgs.llvmPackages_18.lldb
  ];
  # link with mold (rust via the clang driver, meson C/C++ via CC_LD/CXX_LD)
  RUSTFLAGS = "-C link-arg=-fuse-ld=mold";
  CC_LD = "mold";
  CXX_LD = "mold";
  LIBCLANG_PATH = "${pkgs.llvmPackages_18.libclang.lib}/lib";
  LD_LIBRARY_PATH = "./target/debug:${pkgs.wayland}/lib:${pkgs.libGL}/lib:${pkgs.libxkbcommon}/lib";
  G_MESSAGES_DEBUG = "kime";
  GTK_IM_MODULE = "kime";
  GTK_IM_MODULE_FILE = builtins.toString ./.vscode/immodules.cache;
  RUST_BACKTRACE = 1;
}

