{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = github:NixOS/nixpkgs/ff9d9c338734708d3b421516cd0f8618eb3f1ace;
    flake-utils.url = github:numtide/flake-utils;
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem
      (system:
        let pkgs = nixpkgs.legacyPackages.${system}; in
        {
          devShell = import ./shell.nix { inherit pkgs; };
          defaultPackage = import ./default.nix { inherit pkgs; };
        }
      );
}
