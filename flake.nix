{
  description = "A basic flake with a shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    systems.url = "github:nix-systems/default";
    flake-utils = {
      url = "github:numtide/flake-utils";
      inputs.systems.follows = "systems";
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        libPath = builtins.replaceStrings [" "] [":"] (pkgs.lib.makeLibraryPath [
          pkgs.libxkbcommon
          pkgs.libGL
          pkgs.wayland
          pkgs.libX11
          pkgs.libXrandr
          pkgs.libXi
          pkgs.libXcursor
          pkgs.pkg-config
          pkgs.dbus
        ]);
      in {
        devShells.default = pkgs.mkShell {
          packages = [
            (pkgs.rust-bin.selectLatestNightlyWith
              (toolchain:
                toolchain.default.override {
                  extensions = ["rust-src" "llvm-tools-preview" "rustc-codegen-cranelift-preview"];
                }))
            pkgs.cargo-tarpaulin # code coverage
            pkgs.clang
            pkgs.mold
          ];

          WINIT_UNIX_BACKEND = "wayland";
          LD_LIBRARY_PATH = libPath;
          RUSTFLAGS = "-Clink-arg=-Wl,-rpath,${libPath} -Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold -Clink-arg=-flto";

          shellHook = ''
            mkdir -p .nix
            ln -sfn "$(rustc --print sysroot)/bin" .nix/rust-toolchain
            ln -sfn "$(rustc --print sysroot)/lib/rustlib/src/rust" .nix/rust-src
          '';
        };
      }
    );
}
