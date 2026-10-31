{ pkgs ? import <nixpkgs> {} }:
let
  libs = [
    pkgs.libxkbcommon
    pkgs.wayland
    pkgs.vulkan-headers
    pkgs.vulkan-loader
    pkgs.vulkan-tools
    pkgs.vulkan-validation-layers
    pkgs.shaderc
    pkgs.renderdoc
  ];
in
pkgs.mkShell {
  nativeBuildInputs = [  ];
  buildInputs = libs;

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath libs;
  SHADERC_LIB_DIR="${pkgs.shaderc.lib}/lib";
  RUST_BACKTRACE=1;
}
