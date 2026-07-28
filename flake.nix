{
  description = "Korean IME";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs;
    rust-overlay.url = github:oxalica/rust-overlay;
    flake-utils.url = github:numtide/flake-utils;
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          overlays = [ (import rust-overlay) ];
          pkgs = import nixpkgs {
            inherit system overlays;
          };
          rustToolchain = pkgs.rust-bin.stable.latest.default;
        in
        {
          devShells.default = import ./shell.nix { inherit pkgs rustToolchain; };
          # cargo-fuzz needs nightly, and libfuzzer-sys compiles C++, hence
          # the clang stdenv. Pinned by flake.lock through rust-overlay.
          devShells.fuzz =
            let deps = import ./nix/deps.nix { inherit pkgs; };
            in
            (pkgs.mkShell.override { stdenv = pkgs.llvmPackages_18.stdenv; }) {
              name = "kime-fuzz-shell";
              buildInputs = deps.kimeFuzzBuildInputs;
              nativeBuildInputs = deps.kimeFuzzNativeBuildInputs ++ [
                pkgs.rust-bin.nightly.latest.default
              ];
              RUST_BACKTRACE = 1;
            };
          packages.default = import ./default.nix { inherit pkgs rustToolchain; };
        }
      );
}
