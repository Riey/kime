{
  pkgs ? import <nixpkgs> { },
  rustToolchain ? pkgs.symlinkJoin {
    name = "rustc-cargo";
    paths = [
      pkgs.rustc
      pkgs.cargo
    ];
  },
  debug ? false,
  gtk3 ? true,
  gtk4 ? true,
  qt5 ? false,
  qt6 ? true,
}:
let
  src = pkgs.lib.cleanSourceWith {
    src = ./.;
    filter =
      path: type:
      let
        baseName = baseNameOf path;
      in
      pkgs.lib.cleanSourceFilter path type
      && !(baseName == "build" && type == "directory")
      && !(baseName == "target" && type == "directory");
  };
  deps = import ./nix/deps.nix {
    inherit
      pkgs
      gtk3
      gtk4
      qt5
      qt6
      ;
  };
  kimeVersion = pkgs.lib.fileContents ./VERSION;
  cargoProfile = if debug then "debug" else "release";
  boolToFeature = b: if b then "enabled" else "disabled";
  inherit (pkgs) rustPlatform;
  inherit (deps) llvmPackages;
in
llvmPackages.stdenv.mkDerivation {
  name = "kime";
  inherit src;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ [
    rustToolchain
    rustPlatform.cargoSetupHook
    pkgs.makeWrapper
  ];
  version = kimeVersion;
  cargoDeps = rustPlatform.importCargoLock {
    lockFile = ./Cargo.lock;
  };
  LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
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
  postFixup = ''
    substituteInPlace $out/bin/kime-xdg-autostart \
      --replace-fail "/usr/bin/kime" "$out/bin/kime"

    substituteInPlace \
      $out/share/applications/kime.desktop \
      $out/etc/xdg/autostart/kime.desktop \
      --replace-fail "/usr/bin/kime-xdg-autostart" "$out/bin/kime-xdg-autostart"

    wrapProgram $out/bin/kime-wayland \
      --prefix LD_LIBRARY_PATH : "${pkgs.lib.makeLibraryPath [ pkgs.wayland ]}"

    wrapProgram $out/bin/kime-candidate-window \
      --prefix LD_LIBRARY_PATH : "${
        pkgs.lib.makeLibraryPath [
          pkgs.libGL
          pkgs.wayland
          pkgs.libxkbcommon
          pkgs.libxcb
          pkgs.libX11
          pkgs.libXcursor
          pkgs.libXi
          pkgs.libXrandr
        ]
      }"

    wrapProgram $out/bin/kime \
      --prefix PATH : "$out/bin"
  '';
  doCheck = true;
  checkPhase = ''
    cargo test ${if debug then "" else "--release"}
  '';
}
