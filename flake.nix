{
  description = "Dev environment for toy RNN visualization (wgpu + winit on Wayland/X11)";

  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells = {
          default = pkgs.mkShell {
            packages = with pkgs; [
              rustup

              wayland
              libxkbcommon
              xorg.libX11
              xorg.libxcb
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr

              mesa
              vulkan-tools
            ];

            # Make sure dynamic libs are found at runtime (esp. when running via `cargo run`).
            shellHook = ''
              export LD_LIBRARY_PATH=${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:\
              ${pkgs.xorg.libX11}/lib:${pkgs.xorg.libxcb}/lib:${pkgs.xorg.libXcursor}/lib:${pkgs.xorg.libXi}/lib:${pkgs.xorg.libXrandr}/lib:\
              ${pkgs.mesa}/lib:${pkgs.mesa}/lib/dri:${pkgs.vulkan-loader}/lib:\
            '';
          };
        };
      }
    );
}
