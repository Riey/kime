{
  pkgs ? import <nixpkgs> {},
}:
with pkgs;
let
  deps = import ./deps.nix  { pkgs = pkgs; };
in
stdenv.mkDerivation {
  name = "kime-deps-only";
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeBuildInputs;
}
