{
  pkgs ? import <nixpkgs> {},
}:
with pkgs;
let
  deps = import ./deps.nix { pkgs = pkgs; };
in
stdenv.mkDerivation {
  name = "kime-shell";
  dontUseCmakeConfigure = true;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs;
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
}

