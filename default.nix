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
  testArgs = if debug then "" else "--release";
in
llvmPackages_11.stdenv.mkDerivation {
  name = "kime";
  src = gis.gitIgnoreSource ./.;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ [ rustPlatform.cargoSetupHook ];
  version = kimeVersion;
  cargoDeps = rustPlatform.fetchCargoTarball {
    src = gis.gitIgnoreSource ./.;
    sha256 = "0fxfzbb1vm6q6n1k14zin8lklqgfnm40553lch4p5yy0smf6pnd4";
  };
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
  dontUseCmakeConfigure = true;
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
  doCheck = true;
  checkPhase = ''
    cargo test ${testArgs}
  '';
}

