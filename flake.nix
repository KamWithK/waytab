{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    inputs:
    inputs.flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = (import (inputs.nixpkgs) { inherit system; });
      in
      {
        devShell =
          with pkgs;
          pkgs.mkShell {
            buildInputs = [
              # Backend.
              cargo
              rustc
              rustfmt
              taplo
              rustPackages.clippy
              openssl
            ];
            nativeBuildInputs = with pkgs; [
              udev
              clang-tools
              pkg-config
              rustPlatform.bindgenHook
            ];
          };
      }
    );
}
