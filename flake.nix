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
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            rustfmt
            taplo
            rustPackages.clippy
            openssl
            dbus
            udev
            clang-tools
            pkg-config
            rustPlatform.bindgenHook
            wayland
            egl-wayland
            glfw-wayland
            pipewire
            xorg.libxcb
            xorg.libXrandr
            libgbm
            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
            gst_all_1.gst-plugins-good
            gst_all_1.gst-plugins-bad
            gst_all_1.gst-plugins-ugly
            gst_all_1.gst-libav
            gst_all_1.gst-vaapi
            x264
          ];
        };
      }
    );
}
