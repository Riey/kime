{
  pkgs ? import <nixpkgs> {  },
  debug ? false,
}:
let
  deps = import ./nix/deps.nix { pkgs = pkgs; };
  kimeVersion = builtins.readFile ./VERSION;
  testArgs = if debug then "" else "--release";
in
with pkgs;
llvmPackages_13.stdenv.mkDerivation rec {
  name = "kime";
  src = ./.;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ [ rustPlatform.cargoSetupHook ];
  version = kimeVersion;
  cargoDeps = rustPlatform.fetchCargoTarball {
    inherit src;
    sha256 = "sha256-qaBzqJHwFR6sSbrfsNARwBw9wJuR7xvKyqM2/LVZybE=";
  };
  LIBCLANG_PATH = "${pkgs.llvmPackages_13.libclang.lib}/lib";
  dontUseCmakeConfigure = true;
  dontWrapQtApps = true;
  buildPhase = if debug then "bash scripts/build.sh -ad" else "bash scripts/build.sh -ar";
  installPhase = ''
    KIME_BIN_DIR=bin \
    KIME_INSTALL_HEADER=1 \
    KIME_INCLUDE_DIR=include \
    KIME_ICON_DIR=share/icons \
    KIME_LIB_DIR=lib \
    KIME_DOC_DIR=share/doc/kime \
    KIME_QT5_DIR=lib/qt-${pkgs.qt5.qtbase.version} \
    bash scripts/install.sh "$out"
  '';
  doCheck = true;
  checkPhase = ''
    cargo test ${testArgs}
  '';
}

