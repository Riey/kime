{
  packging ? false,
}:
let
  pkgs = import (builtins.fetchTarball {
    url    = "https://github.com/NixOS/nixpkgs/archive/ae09772927566314ad11e366ddb46a9c7ffb666a.tar.gz";
    sha256 = "17dvi648znl87ddcl63f4j59vwd82cggckfnll4c6wgg25q6ygnh";
  }) {};
  deps = import ./nix/deps.nix { pkgs = pkgs; };
in
with pkgs;
llvmPackages_11.stdenv.mkDerivation {
  name = "kime-shell";
  dontUseCmakeConfigure = true;
  buildInputs = deps.kimeBuildInputs;
  nativeBuildInputs = deps.kimeNativeBuildInputs ++ (lib.optionals packging [ gnutar zstd dpkg ]);
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
  RUST_BACKTRACE = 1;
}

