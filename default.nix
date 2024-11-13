{
  pkgs ? import <nixpkgs> {  },
  debug ? false,
}:
let
  src = ./.;
  deps = import ./nix/deps.nix { inherit pkgs; };
  kimeVersion = builtins.readFile ./VERSION;
  testArgs = if debug then "" else "--release";
  inherit (pkgs) llvmPackages_18 rustPlatform qt5;
in
llvmPackages_18.stdenv.mkDerivation {
  name = "kime";
  inherit src;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ [ rustPlatform.cargoSetupHook ];
  version = kimeVersion;
  cargoDeps = rustPlatform.fetchCargoTarball {
    inherit src;
    #hash = "0000000000000000000000000000000000000000000000000000";
    hash = "sha256-2MG6xigiKdvQX8PR457d6AXswTRPRJBPERvZqemjv24=";
  };
  LIBCLANG_PATH = "${llvmPackages_18.libclang.lib}/lib";
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
    KIME_QT5_DIR=lib/qt-${qt5.qtbase.version} \
    bash scripts/install.sh "$out"
  '';
  doCheck = true;
  checkPhase = ''
    cargo test ${testArgs}
  '';
}

