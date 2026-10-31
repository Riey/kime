{
  description = "Korean IME";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      systems = [
        "x86_64-linux"
        "aarch64-linux"
      ];
      forAllSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
    in
    {
      formatter = forAllSystems (system: (pkgsFor system).nixfmt-tree);

      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
        in
        {
          default = import ./default.nix { inherit pkgs rustToolchain; };
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          deps = import ./nix/deps.nix { inherit pkgs; };
        in
        {
          default = import ./shell.nix { inherit pkgs rustToolchain; };
          # cargo-fuzz needs nightly, and libfuzzer-sys compiles C++, hence
          # the clang stdenv. Pinned by flake.lock through rust-overlay.
          fuzz = (pkgs.mkShell.override { stdenv = deps.llvmPackages.stdenv; }) {
            name = "kime-fuzz-shell";
            buildInputs = deps.kimeFuzzBuildInputs;
            nativeBuildInputs = deps.kimeFuzzNativeBuildInputs ++ [
              pkgs.rust-bin.nightly.latest.default
            ];
            RUST_BACKTRACE = 1;
          };
        }
      );
    };
}
