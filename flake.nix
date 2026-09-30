{
  description = "MiniC - A minimal C-like language parser";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            pkgs.cargo
            pkgs.rustc
            pkgs.rustfmt
            pkgs.clippy
            pkgs.rust-analyzer
            pkgs.haskellPackages.shelltestrunner
          ];

          shellHook = ''
            export PS1="MiniC ❄️ \[\033[01;34m\]\w\[\033[00m\] > "
          '';
        };
      }
    );
}
