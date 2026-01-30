{ pkgs, lib, config, inputs, ... }:

let
  libPath = builtins.replaceStrings [" "] [":"] (lib.makeLibraryPath [
    pkgs.libxkbcommon
    pkgs.libGL
    pkgs.wayland
    pkgs.xorg.libX11
    pkgs.xorg.libXrandr
    pkgs.xorg.libXi
    pkgs.xorg.libXcursor
    pkgs.pkg-config
    pkgs.dbus
  ]);
in {
  env.WINIT_UNIX_BACKEND = "wayland";
  env.LD_LIBRARY_PATH = libPath;
  env.PATH = lib.makeBinPath [ pkgs.clang ] + ":${builtins.getEnv "PATH"}";

  packages = [
    pkgs.clang
    pkgs.mold
  ];

  languages.rust = {
    enable = true;
    channel = "nightly";
    components = [ "rustc" "cargo" "clippy" "rustfmt" "rust-analyzer" "rust-src" "llvm-tools-preview" "rustc-codegen-cranelift-preview" ];
    rustflags = "-Clink-arg=-Wl,-rpath,${libPath} -Clink-arg=-fuse-ld=${pkgs.mold}/bin/mold";
  };
}
