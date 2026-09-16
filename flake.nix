{
  description = "Dev environment for Particle-Physics-Sim";

  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
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
              fenix.packages.${system}.stable.toolchain

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
