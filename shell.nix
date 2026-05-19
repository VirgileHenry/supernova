{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = [
    # packages required to run spacecraft
    pkgs.libxkbcommon
    # only required if using wayland
    pkgs.wayland
    # runs the app under vulkan instead of opengl
    pkgs.vulkan-headers
    pkgs.vulkan-loader
    pkgs.vulkan-tools
    pkgs.vulkan-validation-layers
    # shader compilation tools
    pkgs.shaderc

    # tools
    pkgs.renderdoc
  ];
  # add link to libs
  LD_LIBRARY_PATH="${pkgs.libxkbcommon}/lib:${pkgs.wayland}/lib:${pkgs.vulkan-loader}/lib";
  SHADERC_LIB_DIR="${pkgs.shaderc.lib}/lib";
  # configure some rust env vars
  RUST_BACKTRACE=1;
  # env_logger config
  RUST_LOG="info";
}
