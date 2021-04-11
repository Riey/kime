{
  packging ? false,
}:
let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {};
  deps = import ./nix/deps.nix { pkgs = pkgs; };
in
with pkgs;
llvmPackages_11.stdenv.mkDerivation {
  name = "kime-shell";
  dontUseCmakeConfigure = true;
  dontWrapQtApps = true;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ (lib.optionals packging [ gnutar zstd dpkg ]);
  CMAKE_EXPORT_COMPILE_COMMANDS = 1;
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
  RUST_BACKTRACE = 1;
}

