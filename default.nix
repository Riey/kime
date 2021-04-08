{
  pkgs ? import <nixpkgs> {},
  gis ? import (fetchTarball {
    url = https://github.com/icetan/nix-git-ignore-source/archive/v1.0.0.tar.gz;
    sha256 = "1mnpab6x0bnshpp0acddylpa3dslhzd2m1kk3n0k23jqf9ddz57k";
  }) {},
  debug ? false,
}:
with pkgs;
let
  kimeVersion = builtins.readFile ./VERSION;
  deps = import ./nix/deps.nix { pkgs = pkgs; };
in
rustPlatform.buildRustPackage rec {
  src = gis.gitIgnoreSource ./.;
  pname = "kime";
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs;
  version = kimeVersion;
  cargoSha256 = "146m1kg83bmlf6sxi20yawksp47qp0byrh07wkv56vyl5p02fsz1";
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
  dontUseCmakeConfigure = true;
  checkType = if debug then "debug" else "release";
  buildPhase = if debug then "bash scripts/build.sh -ad" else "bash scripts/build.sh -ar";
  installPhase = ''
    KIME_BIN_DIR=bin \
    KIME_INSTALL_HEADER=1 \
    KIME_INCLUDE_DIR=include \
    KIME_ICON_DIR=share/icons \
    KIME_LIB_DIR=lib \
    KIME_QT5_DIR=lib/qt-${pkgs.qt5.qtbase.version} \
    bash scripts/install.sh "$out"
  '';
}

