{
  pkgs ? import <nixpkgs> {},
}:
with pkgs;
let
  deps = import ./nix/deps.nix { pkgs = pkgs; };
in
mkShell {
  name = "kime-shell";
  dontUseCmakeConfigure = true;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs;
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
}

