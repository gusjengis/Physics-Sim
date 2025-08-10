{
  description = "Dev environment for Particle Physics Sim (wgpu + winit on Wayland/X11)";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.05";
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
          # Default shell: Wayland + X11 + Vulkan, good for both backends.
          default = pkgs.mkShell {
            packages = with pkgs; [
              # toolchain / build
              rustup
              cargo
              clang
              cmake
              pkg-config

              # runtime libs winit dlopens at runtime
              wayland
              libxkbcommon
              xorg.libX11
              xorg.libxcb
              xorg.libXcursor
              xorg.libXi
              xorg.libXrandr

              # Vulkan loader + Mesa ICDs (for AMD/Intel or software llvmpipe)
              vulkan-loader
              mesa

              # debug tools
              vulkan-tools # gives `vulkaninfo`
            ];

            # Make sure dynamic libs are found at runtime (esp. when running via `cargo run`)
            shellHook = ''
                          export LD_LIBRARY_PATH=${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:\
              ${pkgs.xorg.libX11}/lib:${pkgs.xorg.libxcb}/lib:${pkgs.xorg.libXcursor}/lib:${pkgs.xorg.libXi}/lib:${pkgs.xorg.libXrandr}/lib:\
              ${pkgs.mesa.drivers}/lib:${pkgs.mesa.drivers}/lib/dri:${pkgs.vulkan-loader}/lib:\
              $LD_LIBRARY_PATH

                          echo
                          echo "🥽 Quick checks:"
                          echo "  - vulkaninfo | head"
                          echo "  - RUST_LOG=wgpu_hal=debug,winit=info WGPU_BACKEND=vulkan cargo run"
                          echo "  - Force X11 for a sanity check: WINIT_UNIX_BACKEND=x11 cargo run"
                          echo

                          # helpful default for debug runs; comment out if noisy:
            '';

            # Let pkg-config see Wayland/X11 if anything uses it at build time
            PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" [
              pkgs.wayland
              pkgs.libxkbcommon
              pkgs.xorg.libX11
              pkgs.xorg.libxcb
              pkgs.xorg.libXcursor
              pkgs.xorg.libXi
              pkgs.xorg.libXrandr
            ];

            RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
          };

          # Convenience shell that forces X11 (sometimes easier to get going first)
          x11 = pkgs.mkShell {
            inputsFrom = [ self.devShells.${system}.default ];
            shellHook = ''
              ${self.devShells.${system}.default.shellHook}
              export WINIT_UNIX_BACKEND=x11
              echo "🔧 Forcing X11 backend (WINIT_UNIX_BACKEND=x11)."
            '';
          };

          # Convenience shell that forces Wayland
          wayland = pkgs.mkShell {
            inputsFrom = [ self.devShells.${system}.default ];
            shellHook = ''
              ${self.devShells.${system}.default.shellHook}
              unset WINIT_UNIX_BACKEND
              echo "🟢 Using Wayland backend (default)."
            '';
          };
        };
      }
    );
}
