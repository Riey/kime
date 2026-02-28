{
  pkgs ? import <nixpkgs> {},
  rustToolchain ? pkgs.rustc,
  debug ? false,
  gtk3 ? true,
  gtk4 ? true,
  qt5 ? false,
  qt6 ? true,
}:
let
  src = pkgs.lib.cleanSourceWith {
    src = ./.;
    filter = path: type:
      let baseName = baseNameOf path;
      in pkgs.lib.cleanSourceFilter path type
      && !(baseName == "build" && type == "directory")
      && !(baseName == "target" && type == "directory");
  };
  deps = import ./nix/deps.nix { inherit pkgs gtk3 gtk4 qt5 qt6; };
  kimeVersion = pkgs.lib.fileContents ./VERSION;
  cargoProfile = if debug then "debug" else "release";
  boolToFeature = b: if b then "enabled" else "disabled";
  inherit (pkgs) llvmPackages_18 rustPlatform;
in
llvmPackages_18.stdenv.mkDerivation {
  name = "kime";
  inherit src;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ [ rustToolchain pkgs.cargo rustPlatform.cargoSetupHook ];
  version = kimeVersion;
  cargoDeps = rustPlatform.fetchCargoVendor {
    inherit src;
    hash = "sha256-ZgWHzXixTZWg7+2nXbw2NjeWD/cskGoZ/VSrM7vCwFs=";
  };
  LIBCLANG_PATH = "${llvmPackages_18.libclang.lib}/lib";
  dontWrapQtApps = true;
  configurePhase = ''
    meson setup build \
      --prefix=$out \
      -Dcargo_profile=${cargoProfile} \
      -Dgtk3=${boolToFeature gtk3} -Dgtk4=${boolToFeature gtk4} \
      -Dqt5=${boolToFeature qt5} -Dqt6=${boolToFeature qt6} \
      ${pkgs.lib.optionalString qt5 "-Dqt5_plugindir=$out/${pkgs.qt5.qtbase.qtPluginPrefix}"} \
      ${pkgs.lib.optionalString qt6 "-Dqt6_plugindir=$out/${pkgs.qt6.qtbase.qtPluginPrefix}"}
  '';
  buildPhase = ''
    ninja -C build
  '';
  installPhase = ''
    ninja -C build install
  '';
  doCheck = true;
  checkPhase = ''
    cargo test ${if debug then "" else "--release"}
  '';
}
