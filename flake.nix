{
  description = "Dev environment for Particle Physics Sim";

  inputs = {
    nixpkgs.url = "nixpkgs/nixos-25.05";
    # pin the flake-compat loader for non-flake commands (optional)
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
      rec {
        devShells.default = pkgs.mkShell {
          # toolchain & build dependencies
          packages = with pkgs; [
            rustup
            cargo
            clang # wgpu build scripts
            pkg-config
            # runtime libs winit tries to dlopen
            wayland
            libxkbcommon
            xorg.libXcursor
            xorg.libXi
            xorg.libXrandr
          ];

          # make the libraries visible at run-time
          shellHook = ''
                        export LD_LIBRARY_PATH=${pkgs.wayland}/lib:${pkgs.libxkbcommon}/lib:\
            ${pkgs.xorg.libXcursor}/lib:${pkgs.xorg.libXi}/lib:${pkgs.xorg.libXrandr}/lib:\
            $LD_LIBRARY_PATH
          '';

          # nice-to-haves
          RUST_SRC_PATH = "${pkgs.rustPlatform.rustLibSrc}";
        };
      }
    );
}
