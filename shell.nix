{
  packaging ? false,
  develop ? false,
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
  nativeBuildInputs = deps.kimeNativeBuildInputs
                      ++ lib.optionals packaging [
                        gnutar
                        zstd
                        dpkg
                      ]
                      ++ lib.optionals develop [
                        rust-analyzer
                        cargo-deny
                        cargo-outdated
                      ];
  CMAKE_EXPORT_COMPILE_COMMANDS = 1;
  LIBCLANG_PATH = "${pkgs.llvmPackages_11.libclang}/lib";
  RUST_BACKTRACE = 1;
}

