{
  pkgs ? import <nixpkgs> {},
  packging ? false,
}:
with pkgs;
let
  deps = import ./nix/deps.nix { pkgs = pkgs; };
in
llvmPackages_11.stdenv.mkDerivation {
  name = "kime-shell";
  dontUseCmakeConfigure = true;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ (lib.optionals packging [ gnutar zstd dpkg ]);
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
  RUST_BACKTRACE = 1;
}

