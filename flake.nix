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
          packages.default = import ./default.nix { inherit pkgs rustToolchain; };
        }
      );
}
