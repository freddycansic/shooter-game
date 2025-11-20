{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";
  };

  outputs = inputs @ {
    self,
    fenix,
    nixpkgs,
    ...
  }: let
    mkPkgs = system:
      import inputs.nixpkgs {
        inherit system;
        overlays = [
          fenix.overlays.default
        ];
      };

    pkgs = mkPkgs "x86_64-linux";

    libPath = with pkgs; lib.makeLibraryPath [
      libxkbcommon
      libGL
      wayland
      xorg.libX11
      xorg.libXrandr
      xorg.libXi
      xorg.libXcursor
      pkg-config
    ];
  in {
    devShells."x86_64-linux".default = pkgs.mkShell {
      packages = with pkgs; [
        (pkgs.fenix.complete.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
          "llvm-tools-preview"
          "rustc-codegen-cranelift-preview"
        ])
        rust-analyzer-nightly
        lld
        clang
        mold
      ];

      RUSTFLAGS = "-Clink-arg=-Wl,-rpath,${libPath} -Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold";

      WINIT_UNIX_BACKEND = "wayland";

      LD_LIBRARY_PATH = libPath;
    };
  };
}
