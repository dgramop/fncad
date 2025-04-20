{
  description = "Build a cargo project without extra checks";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane.url = "github:ipetkov/crane";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.rust-analyzer-src.follows = "";
    };

    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        craneLib = crane.mkLib pkgs;

        dynamicLibs = [] ++ pkgs.lib.optionals pkgs.stdenv.isLinux (with pkgs; [
          libxkbcommon
          vulkan-loader
          xorg.libX11
          xorg.libXcursor
          xorg.libXrandr
          xorg.libxcb
          xorg.libXi
          stdenv.cc.cc.lib
        ]);
        # Common arguments can be set here to avoid repeating them later
        # Note: changes here will rebuild all dependency crates
        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;

          buildInputs = [
            # Add additional build inputs here
          ] ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ] ++ dynamicLibs;
        };

        fncad = craneLib.buildPackage (commonArgs // {
          cargoArtifacts = craneLib.buildDepsOnly commonArgs;
        });
      in
      {
        checks = {
          inherit fncad;
        };

        packages.default = fncad;

        apps.default = flake-utils.lib.mkApp {
          drv = fncad;
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};

          shellHook = ''
            ${if (pkgs.stdenv.isLinux) then "export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath (dynamicLibs ++ [ "/run/opengl-driver" ])}" else ""}
          '';

          packages = [
            pkgs.rust-analyzer
          ];
        };
      });
}
