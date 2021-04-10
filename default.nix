{
  sources ? import ./nix/sources.nix,
  debug ? false,
}:
let
  pkgs = import sources.nixpkgs {};
  gis = import sources.nix-git-ignore-source {};
  deps = import ./nix/deps.nix { pkgs = pkgs; };
  kimeVersion = builtins.readFile ./VERSION;
  testArgs = if debug then "" else "--release";
in
with pkgs;
llvmPackages_11.stdenv.mkDerivation {
  name = "kime";
  src = gis.gitIgnoreSource ./.;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ [ rustPlatform.cargoSetupHook ];
  version = kimeVersion;
  cargoDeps = rustPlatform.fetchCargoTarball {
    src = ./Cargo.lock;
    sha256 = "1ykyd097pwz5xbqxlsq0845pg06g394g1jqwv7ipr6dpbh7r3xqa";
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

